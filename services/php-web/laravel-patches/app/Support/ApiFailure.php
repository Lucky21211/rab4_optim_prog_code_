<?php
namespace App\Support;
use Illuminate\Support\Facades\Log;
use Illuminate\Support\Str;
final class ApiFailure extends \RuntimeException
{
    public function __construct(public readonly string $errorCode, string $message, public readonly ?string $trace = null) { parent::__construct($message); }
    public function envelope(): array {
        $id = $this->trace ?: (string) Str::uuid();
        Log::warning('api_error', ['code' => $this->errorCode, 'trace_id' => $id]);
        return ['ok' => false, 'error' => ['code' => $this->errorCode, 'message' => $this->getMessage(), 'trace_id' => $id]];
    }
}
