<!doctype html>
<html lang="ru">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Space Dashboard</title>
  <link href="/vendor/bootstrap/css/bootstrap.min.css" rel="stylesheet">
  <link rel="stylesheet" href="/vendor/leaflet/leaflet.css"/>
  <style>#map{height:340px}</style>
  <script src="/vendor/leaflet/leaflet.js"></script>
  <script src="/vendor/chart.umd.js"></script>
</head>
<body>
<nav class="navbar navbar-expand-lg bg-body-tertiary mb-3">
  <div class="container">
    <a class="navbar-brand" href="/dashboard">Dashboard</a>
    <a class="nav-link" href="/iss">ISS</a>
    <a class="nav-link" href="/osdr">OSDR</a>
  </div>
</nav>
@yield('content')
<script src="/vendor/bootstrap/js/bootstrap.bundle.min.js"></script>
</body>
</html>
