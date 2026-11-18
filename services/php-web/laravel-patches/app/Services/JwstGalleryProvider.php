<?php
namespace App\Services;
use App\Contracts\GalleryProvider;
final class JwstGalleryProvider implements GalleryProvider {
    public function __construct(private CachedSpaceService $data) {}
    public function feed(array $filters): array { return $this->data->read('/jwst/feed', $filters, config('space.gallery_ttl')); }
}
