set -x
exec mysql --defaults-extra-file=$HOME/IdeaProjects/aelita/stor_diesel/etc/mysql-defaults.ini -h192.168.66.11 edition1 "$@"