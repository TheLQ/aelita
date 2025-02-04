-- Registry > Total

CREATE TABLE `registry_ids`
(
    `xrn`           VARCHAR(100) NOT NULL PRIMARY KEY,
    `published`     VARCHAR(25)  NOT NULL COMMENT "timestamp_rfc3339",
    `publish_cause` TEXT         NOT NULL
);

-- Registry > Links

CREATE TABLE `registry_links`
(
    `xrn_source`    VARCHAR(100) NOT NULL PRIMARY KEY,
    `xrn_target`    VARCHAR(100) NOT NULL,
    `published`     VARCHAR(25)  NOT NULL COMMENT "timestamp_rfc3339",
    `publish_cause` TEXT         NOT NULL
);

-- Project > Names

CREATE TABLE `aproject_names`
(
    `xrn_project_id` INTEGER UNSIGNED NOT NULL PRIMARY KEY,
    `title`          TEXT             NOT NULL,
    `description`    TEXT             NOT NULL,
    `published`      VARCHAR(25)      NOT NULL COMMENT "timestamp_rfc3339",
    `publish_cause`  TEXT             NOT NULL
);

-- Project > Tasks

CREATE TABLE `aproject_lasers`
(
    `xrn_laser_id`  INTEGER UNSIGNED NOT NULL PRIMARY KEY,
    `title`         TEXT             NOT NULL,
    `description`   TEXT             NOT NULL,
    `published`     VARCHAR(25)      NOT NULL COMMENT "timestamp_rfc3339",
    `publish_cause` TEXT             NOT NULL
);

-- Label

CREATE TABLE `alabel_names`
(
    `xrn_label_name` VARCHAR(100) NOT NULL PRIMARY KEY,
    `description`    TEXT         NOT NULL,
    `published`      VARCHAR(25)  NOT NULL COMMENT "timestamp_rfc3339",
    `publish_cause`  TEXT         NOT NULL
);

-- Browser

CREATE TABLE `fire_history`
(
    `xrn_firehist_id` INTEGER UNSIGNED NOT NULL PRIMARY KEY,
    `fire_id`         VARCHAR(13)      NOT NULL COMMENT "browser internal id",
    `fire_last_visit` INTEGER UNSIGNED NOT NULL,
    `visit_count`     INTEGER UNSIGNED NOT NULL,
    `title`           TEXT             NOT NULL,
    `published`       VARCHAR(25)      NOT NULL COMMENT "timestamp_rfc3339",
    `publish_cause`   TEXT             NOT NULL
)

