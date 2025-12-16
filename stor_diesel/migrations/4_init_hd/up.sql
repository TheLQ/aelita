CREATE TABLE IF NOT EXISTS `hd1_sites`
(
    `journal_id`  INTEGER UNSIGNED NOT NULL,
    `hd_site_id`  INTEGER UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    `site_name`   VARCHAR(50)      NOT NULL,
    `description` TEXT             NOT NULL,
    FOREIGN KEY (`journal_id`) REFERENCES `journal_immutable` (`journal_id`)
);

CREATE TABLE IF NOT EXISTS `hd1_galleries`
(
    `journal_id` INTEGER UNSIGNED NOT NULL,
    `hd_site_id` INTEGER UNSIGNED NOT NULL,
    `hd_id`      INTEGER UNSIGNED NOT NULL PRIMARY KEY,
    `tor_hash`   BINARY(50)       NOT NULL,
    FOREIGN KEY (`journal_id`) REFERENCES `journal_immutable` (`journal_id`),
    FOREIGN KEY (`hd_site_id`) REFERENCES `hd1_sites` (`hd_site_id`)
);

CREATE TABLE IF NOT EXISTS `hd1_files_components`
(
    `id`        INTEGER UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    `component` VARBINARY(250)   NOT NULL,
    UNIQUE KEY `comp` (`component`)
);

CREATE TABLE IF NOT EXISTS `hd1_files_parents`
(
    `tree_id`      INTEGER UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    `component_id` INTEGER UNSIGNED NOT NULL,
    `parent_id`    INTEGER UNSIGNED
);
create unique index `glob_unique` on `hd1_files_parents` (`tree_id`, `parent_id`, `component_id`);
create index `parents` on `hd1_files_parents` (`parent_id`);
create index `lookup` on `hd1_files_parents` (`tree_id`, `parent_id`);

CREATE TABLE IF NOT EXISTS `hd1_files_paths`
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
DROP INDEX p0 on `hd1_files_paths`;
DROP INDEX p0_2 on `hd1_files_paths`;
DROP INDEX p0_3 on `hd1_files_paths`;
DROP INDEX p0_4 on `hd1_files_paths`;
