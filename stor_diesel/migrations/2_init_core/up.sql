CREATE TABLE `space_names`
(
    `journal_id`  INTEGER UNSIGNED NOT NULL,
    `space_id`    INTEGER UNSIGNED NOT NULL AUTO_INCREMENT,
    `space_name`  VARCHAR(50)      NOT NULL,
    `description` TEXT             NOT NULL,
    PRIMARY KEY (`space_id`),
    CONSTRAINT `fk_space_names_to_journal`
        FOREIGN KEY (`journal_id`) REFERENCES `journal_immutable` (`journal_id`)
);

CREATE TABLE `space_owned`
(
    `ref_id`      INTEGER UNSIGNED,
    `journal_id`  INTEGER UNSIGNED NOT NULL,
    `space_id`    INTEGER UNSIGNED NOT NULL,
    `child_type1` ENUM ('dummy')   NOT NULL,
    `child_type2` ENUM ('dummy')   NOT NULL,
    `child_id`    INTEGER UNSIGNED NOT NULL,
    `description` TEXT,
    PRIMARY KEY (`ref_id`),
    CONSTRAINT `fk_space_owned_to_journal`
        FOREIGN KEY (`journal_id`) REFERENCES `journal_immutable` (`journal_id`),
    CONSTRAINT `fk_space_owned_to_names`
        FOREIGN KEY (`space_id`) REFERENCES `space_names` (`space_id`),
    UNIQUE KEY `association` (`space_id`, `child_type1`, `child_type2`, `child_id`)
);
show create table `space_owned`
--

