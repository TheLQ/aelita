CREATE TABLE `hd1_sites`
(
    `journal_id`  INTEGER UNSIGNED NOT NULL,
    `hd_site_id`  INTEGER UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    `site_name`   VARCHAR(50)      NOT NULL,
    `description` TEXT             NOT NULL,
    FOREIGN KEY (`journal_id`) REFERENCES `journal_immutable` (`journal_id`)
);

CREATE TABLE `hd1_galleries`
(
    `journal_id` INTEGER UNSIGNED NOT NULL,
    `hd_site_id` INTEGER UNSIGNED NOT NULL,
    `hd_id`      INTEGER UNSIGNED NOT NULL PRIMARY KEY,
    `tor_hash`   BINARY(50)       NOT NULL,
    FOREIGN KEY (`journal_id`) REFERENCES `journal_immutable` (`journal_id`),
    FOREIGN KEY (`hd_site_id`) REFERENCES `hd1_sites` (`hd_site_id`)
);

CREATE TABLE `hd1_files_linear`
(
    `path` VARCHAR(600) NOT NULL PRIMARY KEY
);

CREATE TABLE `hd1_files_tree`
(
    `id`        INTEGER UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    `component` VARBINARY(250)   NOT NULL UNIQUE
);

CREATE TABLE `hd1_files_parents`
(
    `id`       INTEGER UNSIGNED NOT NULL PRIMARY KEY,
    `parentId` INTEGER UNSIGNED,
    INDEX (`parentId`)
);

--

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