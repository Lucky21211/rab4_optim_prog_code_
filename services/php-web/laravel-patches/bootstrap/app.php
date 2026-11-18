<?php
use Illuminate\Foundation\Application;
use Illuminate\Foundation\Configuration\Exceptions;
use Illuminate\Foundation\Configuration\Middleware;
return Application::configure(basePath: dirname(__DIR__))
    ->withRouting(web: __DIR__.'/../routes/web.php', commands: __DIR__.'/../routes/console.php', health: '/up')
    ->withMiddleware(function (Middleware $middleware) { $middleware->append(\App\Http\Middleware\SecurityHeaders::class); })
    ->withExceptions(function (Exceptions $exceptions) {
        $exceptions->render(function (\Throwable $error, \Illuminate\Http\Request $request) {
            if (!$request->is('api/*')) return null;
            if ($error instanceof \App\Support\ApiFailure) $failure = $error;
            elseif ($error instanceof \Illuminate\Validation\ValidationException) $failure = new \App\Support\ApiFailure('VALIDATION_ERROR', 'Некорректные параметры запроса');
            elseif ($error instanceof \Symfony\Component\HttpKernel\Exception\NotFoundHttpException) $failure = new \App\Support\ApiFailure('NOT_FOUND', 'API маршрут не найден');
            else $failure = new \App\Support\ApiFailure('INTERNAL_ERROR', 'Внутренняя ошибка сервиса');
            return response()->json($failure->envelope(), 200);
        });
        // Do not let the default reporter dump HTTP credentials, SQL bindings or DSNs.
        $exceptions->report(function (\Throwable $error) {
            if ($error instanceof \App\Support\ApiFailure) return false;
            \Illuminate\Support\Facades\Log::error('request_failed', ['type' => get_class($error)]);
            return false;
        });
    })
    ->create();
