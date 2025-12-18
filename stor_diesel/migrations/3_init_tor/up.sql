CREATE TABLE `tor1_qb_host`
(
    `qb_host_id` INTEGER UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    `name`       VARCHAR(50)      NOT NULL,
    `address`    VARCHAR(50)      NOT NULL
);

DROP TABLE `tor1_torrents`;
CREATE TABLE `tor1_torrents`
(
    -- @formatter:off for massive enum
    `journal_id`         INTEGER UNSIGNED    NOT NULL,
    `qb_host_id`         INTEGER UNSIGNED    NOT NULL,
    `infohash_v1`        BINARY(20)          NOT NULL,
    `infohash_v2`        BINARY(32)          NOT NULL,
    `name`               TEXT                NOT NULL,
    `comment`            TEXT                NOT NULL,
    `path`               TEXT                NOT NULL,
    `progress`           FLOAT               NOT NULL,
    `original_size`      BIGINT UNSIGNED,
    `selected_size`      BIGINT UNSIGNED,
    `downloaded`         BIGINT UNSIGNED     NOT NULL,
    `uploaded`           BIGINT UNSIGNED     NOT NULL,
    `secs_active`        INTEGER UNSIGNED    NOT NULL,
    `secs_seeding`       INTEGER UNSIGNED    NOT NULL,
    `added_on`           TIMESTAMP           NOT NULL,
    `completion_on`      TIMESTAMP,
    `state`         ENUM('error', 'missingFiles', 'uploading', 'pausedUP', 'queuedUP', 'stalledUP', 'checkingUP', 'forcedUP', 'allocating', 'metaDL', 'downloading', 'pausedDL', 'queuedDL', 'stalledDL', 'checkingDL', 'forcedDL', 'checkingResumeData', 'moving', 'unknown', 'stoppedDL', 'stoppedUP') NOT NULL,
    -- @formatter:on
    PRIMARY KEY (`infohash_v1`),
    CONSTRAINT `fk_tor1_torrents_journal`
        FOREIGN KEY (`journal_id`) REFERENCES `journal_immutable` (`journal_id`)
);
show create table `tor1_torrents`