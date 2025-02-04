-- Registry > Total

CREATE TABLE `jnl_mutation`
(
    `mut_id`        INTEGER UNSIGNED NOT NULL PRIMARY KEY,
    `mut_type`      VARCHAR(40)      NOT NULL,
    `data`          TEXT             NOT NULL,
    `published`     VARCHAR(25)      NOT NULL COMMENT "timestamp_rfc3339",
    `publish_cause` TEXT             NOT NULL
);

CREATE TABLE `jnl_id_counters`
(
    `key`     VARCHAR(40)      NOT NULL PRIMARY KEY,
    `counter` INTEGER UNSIGNED NOT NULL,
    `updated` VARCHAR(25)      NOT NULL COMMENT "timestamp_rfc3339"
);
