-- This file should undo anything in `up.sql`
ALTER TABLE `aproject_names` DROP COLUMN `published`;

ALTER TABLE `xrn_registry` DROP COLUMN `published`;
ALTER TABLE `xrn_registry` ADD COLUMN `published` VARCHAR(100) NOT NULL;

