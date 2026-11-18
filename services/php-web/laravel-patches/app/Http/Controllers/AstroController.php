<?php
namespace App\Http\Controllers;
use Illuminate\Http\Request;
use App\Contracts\EventProvider;
final class AstroController extends Controller {
    public function events(Request $r, EventProvider $events) {
        $filters = $r->validate(['lat' => 'sometimes|numeric|between:-90,90', 'lon' => 'sometimes|numeric|between:-180,180', 'days' => 'sometimes|integer|between:1,30']);
        return response()->json($events->events($filters));
    }
}
