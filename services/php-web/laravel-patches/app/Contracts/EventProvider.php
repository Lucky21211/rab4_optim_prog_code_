<?php
namespace App\Contracts;
interface EventProvider { public function events(array $filters): array; }
