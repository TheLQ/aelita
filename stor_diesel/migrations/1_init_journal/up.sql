CREATE TABLE `publish_log`
(
    `publish_id`        INTEGER UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    `at`                TIMESTAMP        NOT NULL,
    `cause_xrn`         VARCHAR(100),
    `cause_description` TEXT             NOT NULL
);

CREATE TABLE `journal_types`
(
    `journal_type`      INTEGER UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    `journal_type_name` VARCHAR(20)      NOT NULL
);

CREATE TABLE `journal_data_immutable`
(
    `publish_id`   INTEGER UNSIGNED NOT NULL,
    `journal_id`   INTEGER UNSIGNED NOT NULL PRIMARY KEY,
    `journal_type` INTEGER UNSIGNED NOT NULL,
    `data`         BLOB             NOT NULL,
    FOREIGN KEY (`publish_id`) REFERENCES `publish_log` (`publish_id`),
    FOREIGN KEY (`journal_type`) REFERENCES `journal_types` (`journal_type`)
);

CREATE TABLE `journal_data_upgraded`
(
    `publish_id`            INTEGER UNSIGNED NOT NULL,
    `journal_id`            INTEGER UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    `overwrites_journal_id` INTEGER UNSIGNED NOT NULL,
    `journal_type`          INTEGER UNSIGNED NOT NULL,
    `data`                  BLOB             NOT NULL,
    FOREIGN KEY (`publish_id`) REFERENCES `publish_log` (`publish_id`),
    FOREIGN KEY (`journal_type`) REFERENCES `journal_types` (`journal_type`),
    FOREIGN KEY (`overwrites_journal_id`) REFERENCES `journal_data_immutable` (`journal_id`)
);

CREATE TABLE `journal_complete`
(
    `publish_id` INTEGER UNSIGNED NOT NULL,
    `journal_id` INTEGER UNSIGNED NOT NULL PRIMARY KEY,
    FOREIGN KEY (`publish_id`) REFERENCES `publish_log` (`publish_id`)
);
