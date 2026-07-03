### Утилита на Rust для синхронизации метаданных объектов из S3-совместимого бакета в таблицу PostgreSQL. 

Листит объекты постранично и записывает в БД (file_key, bucket, size_bytes, content_type, is_used).
При конфликте по file_key запись не дублируется.
Возвращает число новых вставленных записей.

Работа с S3-совместимыми сервисами (MinIO, AWS S3 и пр.).
Постраничное перечисление объектов через AWS SDK paginator.
Использует sqlx для асинхронных запросов в PostgreSQL.

Конфигурация через переменные окружения / .env.
Простая и идемпотентная логика вставки (ON CONFLICT DO NOTHING).

**Стек**

Rust (рекомендуется stable, указать версию в rust-toolchain при необходимости)
- Cargo
- PostgreSQL (доступный DATABASE_URL)
- S3-совместимый сервис (MinIO/AWS S3)
- .env (рекомендуется dotenvy)

**Переменные окружения**
- DATABASE_URL — строка подключения к PostgreSQL
- S3_ENDPOINT — URL S3-совместимого сервиса (например http://localhost:9000)
- S3_ACCESS_KEY — access key
- S3_SECRET_KEY — secret key
- S3_REGION — регион (любой строкой, используется в конфиге AWS SDK)
- S3_BUCKET — имя бакета для синхронизации

**Структура БД (пример SQL)**
```
CREATE TABLE files (
id SERIAL PRIMARY KEY,
file_key TEXT NOT NULL UNIQUE,
bucket TEXT NOT NULL,
size_bytes BIGINT NOT NULL,
content_type TEXT,
is_used BOOLEAN NOT NULL DEFAULT false,
created_at TIMESTAMP WITH TIME ZONE DEFAULT now());
```

Важно: в коде вставка идёт по полю file_key как уникальному ключу (ON CONFLICT (file_key) DO NOTHING).

**Как запустить локально**
Создайте .env с нужными переменными (см. выше).
Запустите PostgreSQL и S3 (например, MinIO).
Выполните миграцию/создайте таблицу files.

**Сборка и запуск:**
cargo build --release
RUST_LOG=info cargo run --release
(или запустить тестовую функцию init_s3_and_db() и затем sync_bucket_to_db()).

**Ключевые части кода**
init_s3_and_db(): загружает .env, создаёт AWS SDK конфиг с кастомным endpoint и креденшалами, инициализирует Client S3 и sqlx::PgPool, возвращает (Client, PgPool, bucket).
sync_bucket_to_db(s3_client, pool, bucket): листит объекты с помощью paginator, для каждого объекта извлекает key и size и пытается вставить строку в таблицу files; считает количество новых вставок и возвращает его.

**Примечания по реализации и улучшения**
content_type сейчас всегда None; при необходимости можно запросить HeadObject для каждого ключа (но это увеличит количество запросов). 
Альтернативы: хранить content_type в метаданных при загрузке или использовать S3 ListObjectsV2 с дополнительными полями, если SDK/сервер поддерживает.

**Производительность:** текущая реализация выполняет вставку в БД синхронно для каждого объекта. Для ускорения можно:
Собрать батчи и выполнить batch INSERT с ON CONFLICT DO NOTHING.
Использовать асинхронные параллельные задачи для обработки страниц (с контролем степени параллелизма).

**Надёжность:** добавить обработку ошибок для отдельных объектов, логирование пропущенных ключей, ретраи для сетевых ошибок.

**Идемпотентность:** текущая логика безопасна при повторном запуске (ON CONFLICT).

Миграции: интегрировать миграции (sqlx-migrate, refinery или diesel) для гарантированной структуры БД.
Тесты: добавить unit/integration тесты, мок S3 (localstack/мок-клиент) и тестовую базу.

**Пример использования (псевдокод)**
```
let (s3_client, pool, bucket) = init_s3_and_db().await?;
let inserted = sync_bucket_to_db(&s3_client, &pool, &bucket).await?;
println!("Inserted {} new records", inserted);
```


```
flowchart LR
    %% Внешний мир
    U[User Browser] --> F[Frontend Yew WASM]

    %% API‑граница
    F -->|HTTP REST /api/*| GW[API Gateway / Ingress]

    %% Игровые сервисы
    subgraph S["Game Platform (microservices)"]
        direction LR

        G[GameService Axum + Postgres]:::svc
        C[CardsService Axum + Postgres]:::svc
        P[ProfileService Axum + Postgres]:::svc
        ST[StatsService Axum + Postgres]:::svc
    end

    %% Очередь сообщений
    MQ[(Message Queue\nRabbitMQ / Kafka)]

    %% Хранилища
    G -->|SQL| GDB[(games_db)]
    C -->|SQL| CDB[(cards_db)]
    P -->|SQL| PDB[(profiles_db)]
    ST -->|SQL| STDB[(stats_db)]

    %% Взаимодействия между сервисами (HTTP API)
    GW -->|REST /games/*| G
    GW -->|REST /profiles/*| P

    G -->|REST /card-sets/*| C
    G -->|REST /stats/games/*| ST
    P -->|REST /stats/profiles/*| ST

    %% Взаимодействия через очередь (events API)
    G -->|publish GameFinished| MQ
    ST -->|consume GameFinished| MQ
    P -->|consume GameFinished| MQ

    classDef svc fill:#1f2933,stroke:#111,color:#fff;
```
