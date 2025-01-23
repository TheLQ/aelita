-- Your SQL goes here
ALTER TABLE `aproject_names` DROP COLUMN `published`;
ALTER TABLE `aproject_names` ADD COLUMN `published` VARCHAR(25) NOT NULL;

ALTER TABLE `xrn_registry` DROP COLUMN `published`;
ALTER TABLE `xrn_registry` ADD COLUMN `published` VARCHAR(25) NOT NULL;

