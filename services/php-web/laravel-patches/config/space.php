<?php
return [
    'rust_url' => env('RUST_BASE', 'http://rust_iss:3000'),
    'timeout' => (int) env('INTERNAL_TIMEOUT_SECONDS', 45),
    'ttl' => (int) env('READ_CACHE_TTL', 10),
    'gallery_ttl' => (int) env('GALLERY_CACHE_TTL', 120),
    'cms_ttl' => (int) env('CMS_CACHE_TTL', 60),
];
