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

CREATE TABLE `hd1_files_components`
(
    `id`        INTEGER UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    `component` VARBINARY(250)   NOT NULL,
    UNIQUE KEY `comp` (`component`)
);

CREATE TABLE `hd1_files_parents`
(
    `id`        INTEGER UNSIGNED NOT NULL PRIMARY KEY,
    `parent_id` INTEGER UNSIGNED,
    INDEX (`parent_id`)
);

CREATE TABLE `hd1_files_paths`
(
    `path_id` INTEGER UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    `p0`      INTEGER UNSIGNED,
    `p1`      INTEGER UNSIGNED,
    `p2`      INTEGER UNSIGNED,
    `p3`      INTEGER UNSIGNED,
    `p4`      INTEGER UNSIGNED,
    `p5`      INTEGER UNSIGNED,
    `p6`      INTEGER UNSIGNED,
    `p7`      INTEGER UNSIGNED,
    `p8`      INTEGER UNSIGNED,
    `p9`      INTEGER UNSIGNED,
    `p10`     INTEGER UNSIGNED
);
