<?php
namespace App\Contracts;
interface GalleryProvider { public function feed(array $filters): array; }
