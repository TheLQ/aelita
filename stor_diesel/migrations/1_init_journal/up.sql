CREATE TABLE IF NOT EXISTS `journal_immutable`
(
    -- @formatter:off for massive enum
    `journal_id`        INTEGER UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    `journal_type`      ENUM ('Space1', 'QbGetTorJson1', 'NData1') NOT NULL,
    `data`              LONGBLOB         NOT NULL,
    `metadata`          JSON,
    `committed`         BOOLEAN          NOT NULL,
    `at`                TIMESTAMP        NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `cause_xrn`         VARCHAR(100),
    `cause_description` TEXT             NOT NULL
    -- @formatter:on
);
