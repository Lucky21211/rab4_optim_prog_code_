<?php
namespace App\Contracts;
interface SpaceDataSource { public function read(string $path, array $query = []): array; }
