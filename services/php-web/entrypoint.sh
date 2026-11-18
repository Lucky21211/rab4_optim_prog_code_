#!/bin/sh
set -eu
mkdir -p /var/www/html
rsync -a --delete --exclude=storage --exclude=.env /opt/app/ /var/www/html/
cd /var/www/html
touch .env
mkdir -p storage/framework/cache/data storage/framework/sessions storage/framework/views storage/logs bootstrap/cache
chown -R www-data:www-data storage bootstrap/cache
if [ "${SKIP_MIGRATIONS:-false}" != true ]; then
 php artisan migrate --force
fi
echo '{"event":"php_ready"}'
exec "$@"
