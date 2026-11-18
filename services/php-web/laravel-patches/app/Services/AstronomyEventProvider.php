<?php
namespace App\Services;
use App\Contracts\EventProvider;
final class AstronomyEventProvider implements EventProvider {
    public function __construct(private CachedSpaceService $data) {}
    public function events(array $filters): array { return $this->data->read('/astro/events', $filters, config('space.gallery_ttl')); }
}
