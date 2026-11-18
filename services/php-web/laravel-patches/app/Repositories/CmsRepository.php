<?php
namespace App\Repositories;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Cache;
final class CmsRepository {
    public function find(string $slug): ?object {
        return Cache::remember('cms:'.hash('sha256', $slug), config('space.cms_ttl'), fn () => DB::table('cms_blocks')->where('slug', $slug)->where('is_active', true)->first(['title', 'content']));
    }
}
