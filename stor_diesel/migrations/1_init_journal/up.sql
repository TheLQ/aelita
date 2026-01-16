CREATE TABLE IF NOT EXISTS `journal_immutable`
(
    -- @formatter:off for massive enum
    `journal_id`        INTEGER UNSIGNED NOT NULL AUTO_INCREMENT,
    `journal_type`      ENUM ( 'QbGetTorJson1','NData1','ChangeOp1' ) NOT NULL,
    `metadata`          JSON,
    `committed`         BOOLEAN          NOT NULL,
    `at`                TIMESTAMP        NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `cause_xrn`         VARCHAR(100),
    `cause_description` TEXT             NOT NULL,
    `data_hash`         BINARY(32),
    -- @formatter:on
    PRIMARY KEY (`journal_id`)
);

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

# SET foreign_key_checks = 0;
# update journal_immutable
# set journal_id = 1
# where journal_id = 0;
# update journal_immutable_data
# set journal_id = 1
# where journal_id = 0;
# SET foreign_key_checks = 1;

# delete
# from journal_immutable
# where journal_id in (1, 146, 147);

