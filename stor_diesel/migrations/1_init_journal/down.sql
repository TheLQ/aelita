-- This file should undo anything in `up.sql`
SELECT keep_the_journal_pls();
DROP TABLE IF EXISTS `journal_immutable`;
DROP TABLE IF EXISTS `publish_log`;
