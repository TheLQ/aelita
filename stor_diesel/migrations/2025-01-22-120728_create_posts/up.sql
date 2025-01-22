-- Your SQL goes here
CREATE TABLE `aproject_names`(
	`xrn` VARCHAR(100) NOT NULL PRIMARY KEY,
	`title` VARCHAR(100) NOT NULL
);

CREATE TABLE `xrn_registry`(
	`xrn` VARCHAR(100) NOT NULL PRIMARY KEY,
	`published` VARCHAR(100) NOT NULL
);

