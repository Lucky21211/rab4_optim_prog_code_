<?php
namespace App\Http\Controllers;
use Illuminate\Http\Request;
use App\Services\CachedSpaceService;
use App\Support\ApiFailure;
final class OsdrController extends Controller {
    public function index(Request $request, CachedSpaceService $data) {
        $query = $request->validate(['limit' => 'sometimes|integer|between:1,100']);
        try { $items = $data->read('/osdr/list', $query)['items'] ?? []; } catch (ApiFailure $e) { $items = []; }
        foreach ($items as &$item) {
            $url = $item['raw']['REST_URL'] ?? $item['raw']['rest_url'] ?? null;
            $item['rest_url'] = is_string($url) && preg_match('~^https?://~i', $url) ? $url : null;
        }
        unset($item);
        return view('osdr', ['items' => $items, 'src' => 'NASA OSDR']);
    }
}
