<?php
namespace App\Http\Controllers;
use Illuminate\Http\Request;
use App\Services\CachedSpaceService;
final class ProxyController extends Controller {
    public function last(CachedSpaceService $service) { return response()->json($service->read('/last')); }
    public function trend(Request $r, CachedSpaceService $service) {
        $query = $r->validate(['limit' => 'sometimes|integer|between:2,500']);
        return response()->json($service->read('/iss/trend', $query));
    }
}
