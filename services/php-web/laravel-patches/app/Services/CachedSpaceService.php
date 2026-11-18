<?php
namespace App\Services;
use App\Contracts\SpaceDataSource;
use App\Support\ApiFailure;
use Illuminate\Support\Facades\Cache;
final class CachedSpaceService
{
    public function __construct(private SpaceDataSource $source) {}
    public function read(string $path, array $query = [], ?int $ttl = null): array {
        ksort($query);
        $key = 'space:'.hash('sha256', $path.json_encode($query));
        $ttl ??= config('space.ttl');
        try { $hit = Cache::get($key); } catch (\Throwable $e) { $hit = null; }
        if (is_array($hit)) return $hit;
        try {
            return Cache::lock($key.':lock', 60)->block(5, function () use ($key, $path, $query, $ttl) {
                $hit = Cache::get($key);
                if (is_array($hit)) return $hit;
                $data = $this->source->read($path, $query);
                Cache::put($key, $data, max(1, $ttl));
                return $data;
            });
        } catch (ApiFailure $e) { throw $e; }
        catch (\Illuminate\Contracts\Cache\LockTimeoutException $e) { throw new ApiFailure('CACHE_BUSY', 'Данные обновляются, повторите запрос'); }
        catch (\Throwable $e) { return $this->source->read($path, $query); }
    }
}
