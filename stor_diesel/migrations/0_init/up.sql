-- Xrn > Registry

CREATE TABLE `xrn_registry`
(
    `xrn`           VARCHAR(100) NOT NULL PRIMARY KEY,
    `published`     VARCHAR(25)  NOT NULL COMMENT "timestamp_rfc3339",
    `publish_cause` TEXT         NOT NULL
);

-- Project > Names

CREATE TABLE `aproject_names`
(
    `xrn_project_id` INTEGER     NOT NULL PRIMARY KEY AUTO_INCREMENT,
    `title`          TEXT        NOT NULL,
    `description`    TEXT        NOT NULL,
    `published`      VARCHAR(25) NOT NULL COMMENT "timestamp_rfc3339",
    `publish_cause`  TEXT        NOT NULL
);

-- Project > Tasks

CREATE TABLE `aproject_tasks`
(
    `xrn_task_id`   INTEGER     NOT NULL PRIMARY KEY AUTO_INCREMENT,
    `title`         TEXT        NOT NULL,
    `description`   TEXT        NOT NULL,
    `published`     VARCHAR(25) NOT NULL COMMENT "timestamp_rfc3339",
    `publish_cause` TEXT        NOT NULL
);

CREATE TABLE `aproject_tasks_map`
(
    `xrn_task_id`   INTEGER     NOT NULL PRIMARY KEY,
    `xrn`           INTEGER     NOT NULL,
    `published`     VARCHAR(25) NOT NULL COMMENT "timestamp_rfc3339",
    `publish_cause` TEXT        NOT NULL,
    FOREIGN KEY (xrn_task_id)
        REFERENCES aproject_tasks (xrn_task_id) ON DELETE RESTRICT,
    FOREIGN KEY (xrn)
        REFERENCES xrn_registry (xrn) ON DELETE RESTRICT
);

-- Label > Names

CREATE TABLE `alabel_names`
(
    `xrn_label_name` VARCHAR(100) NOT NULL PRIMARY KEY,
    `description`    TEXT         NOT NULL,
    `published`      VARCHAR(25)  NOT NULL COMMENT "timestamp_rfc3339",
    `publish_cause`  TEXT         NOT NULL
);

CREATE TABLE `alabel_names_map`
(
    `xrn_label_name` VARCHAR(100) NOT NULL PRIMARY KEY,
    `xrn`            INTEGER      NOT NULL,
    `published`      VARCHAR(25)  NOT NULL COMMENT "timestamp_rfc3339",
    `publish_cause`  TEXT         NOT NULL,
    FOREIGN KEY (xrn_label_name)
        REFERENCES alabel_names (xrn_label_name) ON DELETE RESTRICT,
    FOREIGN KEY (xrn)
        REFERENCES xrn_registry (xrn) ON DELETE RESTRICT
);

