CREATE TABLE IF NOT EXISTS `hd1_sites`
(
    `journal_id`  INTEGER UNSIGNED NOT NULL,
    `hd_site_id`  INTEGER UNSIGNED NOT NULL AUTO_INCREMENT,
    `site_name`   VARCHAR(50)      NOT NULL,
    `description` TEXT             NOT NULL,
    PRIMARY KEY (`hd_site_id`),
    CONSTRAINT `fk_hd1_sites_journal`
        FOREIGN KEY (`journal_id`) REFERENCES `journal_immutable` (`journal_id`)
);

CREATE TABLE IF NOT EXISTS `hd1_galleries`
(
    `journal_id` INTEGER UNSIGNED NOT NULL,
    `hd_site_id` INTEGER UNSIGNED NOT NULL,
    `hd_id`      INTEGER UNSIGNED NOT NULL,
    `tor_hash`   BINARY(50)       NOT NULL,
    PRIMARY KEY (`hd_id`),
    CONSTRAINT `fk_hd1_galleries_journal`
        FOREIGN KEY (`journal_id`) REFERENCES `journal_immutable` (`journal_id`),
    CONSTRAINT `fk_hd1_galleries_sites`
        FOREIGN KEY (`hd_site_id`) REFERENCES `hd1_sites` (`hd_site_id`)
);

CREATE TABLE IF NOT EXISTS `hd1_files_components`
(
    `id`        INTEGER UNSIGNED NOT NULL AUTO_INCREMENT,
    `component` VARBINARY(250)   NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `comp` (`component`)
);

# DROP TABLE `hd1_files_parents`
CREATE TABLE IF NOT EXISTS `hd1_files_parents`
(
    `tree_id`      INTEGER UNSIGNED NOT NULL,
    `tree_depth`   INTEGER UNSIGNED NOT NULL,
    `component_id` INTEGER UNSIGNED NOT NULL,
    `parent_id`    INTEGER UNSIGNED,
    `created`      TIMESTAMP        NOT NULL,
    `modified`     TIMESTAMP        NOT NULL,
    `size`         BIGINT UNSIGNED  NOT NULL,
    `user_id`      INTEGER UNSIGNED NOT NULL,
    `group_id`     INTEGER UNSIGNED NOT NULL,
    `hard_links`   BIGINT UNSIGNED  NOT NULL,
    PRIMARY KEY (`tree_id`),
    UNIQUE KEY `glob_unique` (`tree_depth`, `parent_id`, `component_id`),
    CONSTRAINT `fk_hd1_files_parents_components`
        FOREIGN KEY (`component_id`) REFERENCES `hd1_files_components` (`id`)
);
# show create table `hd1_files_parents`;

CREATE TABLE IF NOT EXISTS `hd1_files_links`
(
    `at_tree`     INTEGER UNSIGNED NOT NULL,
    `target_tree` INTEGER UNSIGNED NOT NULL,
    PRIMARY KEY (`at_tree`),
    CONSTRAINT `fk_hd1_files_links_source`
        FOREIGN KEY (`at_tree`) REFERENCES `hd1_files_parents` (`tree_id`),
    CONSTRAINT `fk_hd1_files_links_target`
        FOREIGN KEY (`target_tree`) REFERENCES `hd1_files_parents` (`tree_id`)
);
# drop table hd1_files_links;

CREATE TABLE IF NOT EXISTS `hd1_files_paths`
(
    `path_id` INTEGER UNSIGNED NOT NULL AUTO_INCREMENT,
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
    `p10`     INTEGER UNSIGNED,
    PRIMARY KEY (`path_id`),
    KEY `i0` (`p0`),
    KEY `i1` (`p1`),
    KEY `i2` (`p2`),
    KEY `i3` (`p3`),
    KEY `i4` (`p4`),
    KEY `i5` (`p5`),
    KEY `i6` (`p6`),
    KEY `i7` (`p7`)
);
# create index `i0` on `hd1_files_paths` (`p0`);
# create index `i1` on `hd1_files_paths` (`p1`);
# create index `i2` on `hd1_files_paths` (`p2`);
# create index `i3` on `hd1_files_paths` (`p3`);
# create index `i4` on `hd1_files_paths` (`p4`);
# create index `i5` on `hd1_files_paths` (`p5`);
# create index `i6` on `hd1_files_paths` (`p6`);
# create index `i7` on `hd1_files_paths` (`p7`);
show create table `hd1_files_paths`;

CREATE TABLE `hd1_roots`
(
    `space_id` INTEGER UNSIGNED NOT NULL,
    `rtype`    ENUM ('')        NOT NULL,
    PRIMARY KEY (`space_id`),
    CONSTRAINT `fk_hd1_roots_space`
        FOREIGN KEY (`space_id`) REFERENCES `space_names` (`space_id`)
);
ALTER TABLE `hd1_roots`
    MODIFY `rtype`
        ENUM ('ZfsDataset', 'Project')