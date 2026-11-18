FROM node:22-alpine AS assets
WORKDIR /assets
RUN npm init -y && npm install --save-exact bootstrap@5.3.3 leaflet@1.9.4 chart.js@4.4.7
FROM php:8.3-fpm-alpine AS application
RUN apk add --no-cache bash curl git unzip rsync postgresql-dev libxml2-dev oniguruma-dev \
 && docker-php-ext-install pdo_pgsql dom mbstring
COPY --from=composer:2 /usr/bin/composer /usr/local/bin/composer
ENV COMPOSER_ALLOW_SUPERUSER=1
WORKDIR /opt
RUN composer create-project --no-interaction --prefer-dist laravel/laravel:^12.0 app \
 && cd app && composer require --no-interaction predis/predis:^2.3 \
 && rm -f .env
WORKDIR /opt/app
COPY laravel-patches/ ./
COPY --from=assets /assets/node_modules/bootstrap/dist/ ./public/vendor/bootstrap/
COPY --from=assets /assets/node_modules/leaflet/dist/ ./public/vendor/leaflet/
COPY --from=assets /assets/node_modules/chart.js/dist/chart.umd.js ./public/vendor/chart.umd.js
RUN rm -f tests/Feature/ExampleTest.php tests/Unit/ExampleTest.php
RUN composer dump-autoload --optimize
COPY entrypoint.sh /usr/local/bin/php_entrypoint.sh
RUN chmod +x /usr/local/bin/php_entrypoint.sh
WORKDIR /var/www/html
ENTRYPOINT ["/usr/local/bin/php_entrypoint.sh"]
CMD ["php-fpm", "-F"]
