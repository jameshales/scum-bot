#!/bin/bash

mkdir -p $(dirname "$DATABASE_PATH")

if [ ! -f "$DATABASE_PATH" ]; then
  db_path=$(mktemp -d)
  cat /opt/scum-bot/share/sql/*.sql | sqlite3 $db_path/scum-bot.db
  mv -n $db_path/scum-bot.db "$DATABASE_PATH"
  rmdir $db_path
fi

cd /opt/scum-bot/var/

exec /opt/scum-bot/bin/scum_bot
