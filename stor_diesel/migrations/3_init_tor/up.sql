CREATE TABLE `tor1_qb_host`
(
    `qb_host_id` INTEGER UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    `name`       VARCHAR(50)      NOT NULL,
    `address`    VARCHAR(50)      NOT NULL
);

CREATE TABLE `tor1_torrents`
(
    -- @formatter:off for massive enum
    `journal_id`         INTEGER UNSIGNED    NOT NULL,
    `torhash`            BINARY(20)          NOT NULL PRIMARY KEY,
    `tor_status_changed` TIMESTAMP           NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `qb_host_id`         INTEGER UNSIGNED    NOT NULL,
    `tor_status`         ENUM('error', 'missingFiles', 'uploading', 'pausedUP', 'queuedUP', 'stalledUP', 'checkingUP', 'forcedUP', 'allocating', 'metaDL', 'downloading', 'pausedDL', 'queuedDL', 'stalledDL', 'checkingDL', 'forcedDL', 'checkingResumeData', 'moving', 'unknown', 'stoppedDL', 'stoppedUP') NOT NULL,
    -- @formatter:on
    FOREIGN KEY (`journal_id`) REFERENCES `journal_immutable` (`journal_id`)
);