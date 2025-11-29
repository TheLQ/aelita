-- This file should undo anything in `up.sql`
DROP TABLE IF EXISTS `publish_log`;
DROP TABLE IF EXISTS `journal_types`;
DROP TABLE IF EXISTS `journal_data_immutable`;
DROP TABLE IF EXISTS `journal_data_upgraded`;
DROP TABLE IF EXISTS `journal_complete`;
