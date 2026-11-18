<?php
namespace App\Clients;
use App\Contracts\SpaceDataSource;
use App\Support\ApiFailure;
use Illuminate\Support\Facades\Http;
final class RustClient implements SpaceDataSource
{
    public function read(string $path, array $query = []): array {
        try {
            // Retries belong to the external client in Rust, not every proxy layer.
            $response = Http::baseUrl(config('space.rust_url'))->acceptJson()
                ->connectTimeout(3)->timeout(config('space.timeout'))->get($path, $query);
        } catch (\Illuminate\Http\Client\ConnectionException $e) {
            throw new ApiFailure('UPSTREAM_UNAVAILABLE', 'Сервис данных временно недоступен');
        }
        if (!$response->successful()) throw new ApiFailure('UPSTREAM_'.$response->status(), 'Сервис данных вернул ошибку');
        $data = $response->json();
        if (!is_array($data)) throw new ApiFailure('UPSTREAM_JSON', 'Некорректный ответ сервиса данных');
        if (($data['ok'] ?? true) === false) throw new ApiFailure($data['error']['code'] ?? 'UPSTREAM_ERROR', $data['error']['message'] ?? 'Ошибка источника', $data['error']['trace_id'] ?? null);
        return $data;
    }
}
