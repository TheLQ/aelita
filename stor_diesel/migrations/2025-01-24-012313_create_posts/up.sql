-- Your SQL goes here
CREATE TABLE `aproject_names`(
	`xrn_project_id` INTEGER NOT NULL PRIMARY KEY,
	`title` TEXT NOT NULL,
	`published` VARCHAR(25) NOT NULL
);

CREATE TABLE `aproject_tasks`(
	`xrn_task_id` INTEGER NOT NULL PRIMARY KEY,
	`title` TEXT NOT NULL,
	`published` VARCHAR(25) NOT NULL
);

CREATE TABLE `aproject_tasks_map`(
	`xrn_project_id` INTEGER NOT NULL,
	`xrn_task_id` INTEGER NOT NULL PRIMARY KEY,
	`published` VARCHAR(25) NOT NULL
);

CREATE TABLE `xrn_registry`(
	`xrn` VARCHAR(100) NOT NULL PRIMARY KEY,
	`published` VARCHAR(25) NOT NULL
);

