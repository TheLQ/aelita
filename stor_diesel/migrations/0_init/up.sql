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
    `xrn_project_id` INTEGER UNSIGNED NOT NULL PRIMARY KEY AUTO_INCREMENT,
    `title`          TEXT             NOT NULL,
    `description`    TEXT             NOT NULL,
    `published`      VARCHAR(25)      NOT NULL COMMENT "timestamp_rfc3339",
    `publish_cause`  TEXT             NOT NULL
);

-- Project > Tasks

CREATE TABLE `aproject_tasks`
(
    `xrn_task_id`   INTEGER UNSIGNED NOT NULL PRIMARY KEY AUTO_INCREMENT,
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

