CREATE TABLE `publish_log`
(
    `publish_id`        INTEGER UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    `at`                TIMESTAMP        NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `cause_xrn`         VARCHAR(100),
    `cause_description` TEXT             NOT NULL
);

CREATE TABLE `journal_immutable`
(
    -- @formatter:off for massive enum
    `publish_id`   INTEGER UNSIGNED NOT NULL,
    `journal_id`   INTEGER UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    `data`         BLOB             NOT NULL,
    `committed`    BOOLEAN          NOT NULL,
    `journal_type` ENUM ('Journal1', 'Space1') NOT NULL,
    -- @formatter:on
    FOREIGN KEY (`publish_id`) REFERENCES `publish_log` (`publish_id`)
);
