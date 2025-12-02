CREATE TABLE `hd1_sites`
(
    `publish_id`  INTEGER UNSIGNED NOT NULL,
    `hd_site_id`  INTEGER UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    `site_name`   VARCHAR(50)      NOT NULL,
    `description` TEXT             NOT NULL,
    FOREIGN KEY (`publish_id`) REFERENCES `publish_log` (`publish_id`)
);

CREATE TABLE `hd1_galleries`
(
    `publish_id` INTEGER UNSIGNED NOT NULL,
    `space_id`   INTEGER UNSIGNED NOT NULL,
    `hd_site_id` INTEGER UNSIGNED NOT NULL,
    `hd_id`      INTEGER UNSIGNED NOT NULL PRIMARY KEY,
    `tor_hash`   BINARY(50)       NOT NULL,
    FOREIGN KEY (`publish_id`) REFERENCES `publish_log` (`publish_id`),
    FOREIGN KEY (`space_id`) REFERENCES `space_names` (`space_id`),
    FOREIGN KEY (`hd_site_id`) REFERENCES `hd1_sites` (`hd_site_id`)
);

--

CREATE TABLE `tor1_status_types`
(
    `tor_status_type` INTEGER UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    `name`            VARCHAR(50)      NOT NULL
);

CREATE TABLE `tor1_qb_host`
(
    `publish_id` INTEGER UNSIGNED NOT NULL,
    `qb_host_id` INTEGER UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    `name`       VARCHAR(50)      NOT NULL,
    FOREIGN KEY (`publish_id`) REFERENCES `publish_log` (`publish_id`)
);

CREATE TABLE `tor1_torrents`
(
    `publish_id`         INTEGER UNSIGNED NOT NULL,
    `space_id`           INTEGER UNSIGNED NOT NULL,
    `torhash`            BINARY(50)       NOT NULL PRIMARY KEY,
    `tor_status_type`    INTEGER UNSIGNED NOT NULL,
    `tor_status_changed` TIMESTAMP        NOT NULL,
    `qb_host_id`         INTEGER UNSIGNED NOT NULL,
    FOREIGN KEY (`publish_id`) REFERENCES `publish_log` (`publish_id`),
    FOREIGN KEY (`space_id`) REFERENCES `space_names` (`space_id`)
);


