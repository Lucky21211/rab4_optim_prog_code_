<?php
namespace App\Http\Middleware;
use App\Support\ApiFailure;
use Illuminate\Validation\ValidationException;
final class ApiEnvelope {
    public function handle($request, \Closure $next) {
        try { return $next($request); }
        catch (ApiFailure $e) { return response()->json($e->envelope(), 200); }
        catch (ValidationException $e) { return response()->json((new ApiFailure('VALIDATION_ERROR', 'Некорректные параметры запроса'))->envelope(), 200); }
        catch (\Throwable $e) { return response()->json((new ApiFailure('INTERNAL_ERROR', 'Внутренняя ошибка сервиса'))->envelope(), 200); }
    }
}
