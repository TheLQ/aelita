CREATE TABLE IF NOT EXISTS `journal_immutable`
(
    -- @formatter:off for massive enum
    `journal_id`        INTEGER UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    `journal_type`      ENUM ('Space1', 'QbGetTorJson1', 'NData1') NOT NULL,
    `metadata`          JSON,
    `committed`         BOOLEAN          NOT NULL,
    `at`                TIMESTAMP        NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `cause_xrn`         VARCHAR(100),
    `cause_description` TEXT             NOT NULL,
    `data_hash`         BINARY(32)
    -- @formatter:on
);
# ALTER TABLE journal_immutable
#     ADD COLUMN data_hash BINARY(32);

CREATE TABLE IF NOT EXISTS `journal_immutable_data`
(
    `journal_id` INTEGER UNSIGNED NOT NULL,
    `data`       LONGBLOB         NOT NULL,
    `data_id`    INTEGER UNSIGNED NOT NULL AUTO_INCREMENT,
    PRIMARY KEY (`data_id`),
    CONSTRAINT `fk_journal_immutable_data_journal`
        FOREIGN KEY (`journal_id`) REFERENCES `journal_immutable` (`journal_id`)
);
# ALTER TABLE `journal_immutable_data`
#     ADD CONSTRAINT `fk_journal_immutable_data_journal`
#         FOREIGN KEY (`journal_id`) REFERENCES `journal_immutable` (`journal_id`)
#     ADD COLUMN (`data_id` INTEGER UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY);