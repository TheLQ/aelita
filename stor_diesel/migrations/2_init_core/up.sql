CREATE TABLE `space_names`
(
    `journal_id`  INTEGER UNSIGNED NOT NULL,
    `space_id`    INTEGER UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    `space_name`  VARCHAR(50)      NOT NULL,
    `description` TEXT             NOT NULL,
    FOREIGN KEY (`journal_id`) REFERENCES `journal_immutable` (`journal_id`)
);

CREATE TABLE `space_owned`
(
    `journal_id`  INTEGER UNSIGNED NOT NULL,
    `space_id`    INTEGER UNSIGNED NOT NULL PRIMARY KEY,
    `child_xrn`   VARCHAR(100)     NOT NULL,
    `description` TEXT             NOT NULL,
    FOREIGN KEY (`journal_id`) REFERENCES `journal_immutable` (`journal_id`),
    FOREIGN KEY (`space_id`) REFERENCES `space_names` (`space_id`),
    UNIQUE KEY `association` (`space_id`, `child_xrn`)
);

--

