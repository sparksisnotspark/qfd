param(
  [string]$BaseUrl = "http://127.0.0.1:8080",
  [string]$MySqlBin = "D:\phpstudy_pro\Extensions\MySQL5.7.26\bin\mysql.exe",
  [string]$MySqlUser = "root",
  [string]$MySqlPassword = "root",
  [string]$MySqlDatabase = "dy_server",
  [string]$AppKey = "6SDXSF6E",
  [string]$CaijiUser = "312790696@qq.com",
  [string]$CaijiPassword = "zhuxueyi"
)

$ErrorActionPreference = "Stop"

function Info($msg) {
  Write-Host "[INFO] $msg"
}

function Assert($condition, $message) {
  if ($null -eq $message -and $condition -is [System.Array] -and $condition.Count -ge 2) {
    $message = [string]$condition[1]
    $condition = [bool]$condition[0]
  }
  if (-not $condition) {
    $text = [string]$message
    if ([string]::IsNullOrWhiteSpace($text)) {
      $text = "assertion failed"
    }
    Write-Host "[FAIL] $text" -ForegroundColor Red
    throw $text
  }
}

function Invoke-JsonRequest {
  param(
    [string]$Method,
    [string]$Url,
    [object]$Body = $null,
    [hashtable]$Headers = $null,
    $WebSession = $null,
    [switch]$Json
  )

  $params = @{
    Method          = $Method
    Uri             = $Url
    UseBasicParsing = $true
  }
  if ($Headers) { $params.Headers = $Headers }
  if ($WebSession) { $params.WebSession = $WebSession }

  if ($null -ne $Body) {
    if ($Json) {
      $params.ContentType = "application/json"
      $params.Body = ($Body | ConvertTo-Json -Depth 8)
    } else {
      $params.Body = $Body
    }
  }

  $resp = Invoke-WebRequest @params 2>$null
  return ($resp.Content | ConvertFrom-Json)
}

function Try-JsonRequest {
  param(
    [string]$Name,
    [scriptblock]$Block
  )
  try {
    return & $Block
  } catch {
    Write-Host "[FAIL] request $Name failed: $($_.Exception.Message)" -ForegroundColor Red
    throw
  }
}

function Invoke-MySqlScalar {
  param([string]$Sql)
  $args = @(
    "-N",
    "-u$MySqlUser",
    "-p$MySqlPassword",
    "-D", $MySqlDatabase,
    "-e", $Sql
  )
  $output = & $MySqlBin @args
  return ((@($output) | Where-Object { $_ -and $_ -notmatch '^mysql: \[Warning\]' }) | Out-String).Trim()
}

function Invoke-MySqlRows {
  param([string]$Sql)
  $args = @(
    "-N",
    "-u$MySqlUser",
    "-p$MySqlPassword",
    "-D", $MySqlDatabase,
    "-e", $Sql
  )
  $output = & $MySqlBin @args
  return @($output | ForEach-Object { ($_ | Out-String).Trim() } | Where-Object { $_ -ne "" -and $_ -notmatch '^mysql: \[Warning\]' })
}

function Start-Server {
  $logPath = Join-Path $env:TEMP "dy_server_go_verify.log"
  if (Test-Path $logPath) { Remove-Item $logPath -Force }

  Get-NetTCPConnection -LocalPort 8080 -State Listen -ErrorAction SilentlyContinue |
    Select-Object -ExpandProperty OwningProcess -Unique |
    ForEach-Object { Stop-Process -Id $_ -Force }

  $process = Start-Process -FilePath powershell -ArgumentList '-NoProfile','-Command',"cd D:\dy-server-go\server; go run ./cmd/api *> '$logPath'" -PassThru -WindowStyle Hidden

  for ($i = 0; $i -lt 30; $i++) {
    Start-Sleep -Milliseconds 500
    try {
      $version = Invoke-WebRequest -UseBasicParsing "$BaseUrl/app/version.json"
      if ($version.StatusCode -eq 200) {
        return @{
          Process = $process
          LogPath = $logPath
        }
      }
    } catch {}
  }

  if (Test-Path $logPath) {
    Get-Content $logPath | Write-Host
  }
  throw "server not ready"
}

function Stop-Server($server) {
  if ($server -and $server.Process -and -not $server.Process.HasExited) {
    Stop-Process -Id $server.Process.Id -Force
  }
}

Info "building server"
Push-Location D:\dy-server-go\server
go build ./...
Pop-Location

Info "building web"
Push-Location D:\dy-server-go\web
cmd /c "pnpm build >nul 2>nul"
if ($LASTEXITCODE -ne 0) {
  throw "web build failed"
}
Pop-Location

$server = $null
try {
  Info "starting server"
  $server = Start-Server

  Info "checking public routes"
  $version = Try-JsonRequest "version" { Invoke-JsonRequest -Method GET -Url "$BaseUrl/app/version.json" }
  Info "version route ok"
  Write-Host "[DEBUG] version payload: $($version | ConvertTo-Json -Compress)"
  Assert ($null -ne $version.version) "version.json missing version"

  $running = Try-JsonRequest "getRunningStatus" { Invoke-JsonRequest -Method GET -Url "$BaseUrl/api/getRunningStatus" }
  Info "getRunningStatus route ok"
  Write-Host "[DEBUG] running payload: $($running | ConvertTo-Json -Compress)"
  Assert ($running.code -eq 0) "getRunningStatus failed"

  $token = Try-JsonRequest "tokenExpiredAt" { Invoke-JsonRequest -Method POST -Url "$BaseUrl/auth/tokenExpiredAt" -Body @{ keycode = $AppKey } }
  Info "tokenExpiredAt route ok"
  Write-Host "[DEBUG] token payload: $($token | ConvertTo-Json -Compress)"
  Assert ($token.code -eq 0) "tokenExpiredAt failed"

  Info "checking caiji login"
  $session = New-Object Microsoft.PowerShell.Commands.WebRequestSession
  $login = Try-JsonRequest "caiji/login" { Invoke-JsonRequest -Method POST -Url "$BaseUrl/caiji/login" -Body @{ username = $CaijiUser; password = $CaijiPassword } -WebSession $session }
  Info "caiji login ok"
  Assert ($login.code -eq 0) "caiji login failed"

  $sampleCaijiRow = Invoke-MySqlRows "select id, cj_name from dy_caiji order by id asc limit 1;"
  Assert ($sampleCaijiRow.Count -ge 1) "no dy_caiji sample row found"
  $sampleCaijiParts = $sampleCaijiRow[0] -split "`t", 2
  $sampleCaijiId = [int]$sampleCaijiParts[0]
  $sampleCaijiName = $sampleCaijiParts[1]

  $sampleUserRows = Invoke-MySqlRows "select id, username from users order by id asc limit 2;"
  Assert ($sampleUserRows.Count -ge 1) "no users sample row found"
  $sampleUserParts = $sampleUserRows[0] -split "`t", 2
  $sampleUserId = [int]$sampleUserParts[0]
  $sampleUsername = $sampleUserParts[1]
  $conflictUsername = $null
  if ($sampleUserRows.Count -ge 2) {
    $conflictUserParts = $sampleUserRows[1] -split "`t", 2
    $conflictUsername = $conflictUserParts[1]
  }

  $cookieList = Try-JsonRequest "caiji/cookieList" { Invoke-JsonRequest -Method GET -Url "$BaseUrl/caiji/cookieList" -WebSession $session }
  Info "cookieList ok"
  Assert ($cookieList.code -eq 0) "cookieList failed"

  Info "checking add cookie and caijiGetCookie"
  $cookieName = "verify-cookie-" + [DateTimeOffset]::Now.ToUnixTimeSeconds()
  $cookieValue = "sessionid=verify; sid_guard=verify;"
  $addCookie = Invoke-JsonRequest -Method POST -Url "$BaseUrl/caiji/addCookie" -Body @{
    dy_name = $cookieName
    cookie  = $cookieValue
  } -WebSession $session
  Info "addCookie ok"
  Assert ($addCookie.code -eq 0) "addCookie failed"

  $cookieId = Invoke-MySqlScalar "select id from dy_cookies where dy_name = '$cookieName' limit 1;"
  Assert ($cookieId -match '^\d+$') "added cookie not found in db"

  $allocateVerifyCaiji = Invoke-JsonRequest -Method POST -Url "$BaseUrl/caiji/allocateCookie" -Body @{ caiji = @($sampleCaijiId); cookie = @([int]$cookieId) } -WebSession $session -Json
  Assert ($allocateVerifyCaiji.code -eq 0) "allocateCookie for caijiGetCookie failed"

  $caijiCookie = Try-JsonRequest "caiji/caijiGetCookie" { Invoke-JsonRequest -Method POST -Url "$BaseUrl/caiji/caijiGetCookie" -Body @{ cj_name = $sampleCaijiName } -WebSession $session }
  Info "caijiGetCookie ok"
  Assert ($caijiCookie.code -eq 0) "caijiGetCookie failed"
  Assert (($caijiCookie.data.cj_name -eq $sampleCaijiName) -or ($caijiCookie.data.cj_name -eq $null)) "caijiGetCookie returned unexpected caiji"

  $keysList = Try-JsonRequest "caiji/keysManageList" { Invoke-JsonRequest -Method GET -Url "$BaseUrl/caiji/keysManageList?page=1&page_size=2" -WebSession $session }
  Info "keysManageList ok"
  Assert ($keysList.code -eq 0) "keysManageList failed"

  $summary = Try-JsonRequest "caiji/adminSummary" { Invoke-JsonRequest -Method GET -Url "$BaseUrl/caiji/adminSummary" -WebSession $session }
  Info "adminSummary ok"
  Assert ($summary.code -eq 0) "adminSummary failed"

  $usersList = Try-JsonRequest "caiji/usersList" { Invoke-JsonRequest -Method GET -Url "$BaseUrl/caiji/usersList?page=1&page_size=2" -WebSession $session }
  Info "usersList ok"
  Assert ($usersList.code -eq 0) "usersList failed"

  $roomUrlList = Try-JsonRequest "caiji/roomUrlList" { Invoke-JsonRequest -Method GET -Url "$BaseUrl/caiji/roomUrlList?page=1&page_size=2" -WebSession $session }
  Info "roomUrlList ok"
  Assert ($roomUrlList.code -eq 0) "roomUrlList failed"

  $keyStoreList = Try-JsonRequest "caiji/keyStoreList" { Invoke-JsonRequest -Method GET -Url "$BaseUrl/caiji/keyStoreList?page=1&page_size=2" -WebSession $session }
  Info "keyStoreList ok"
  Assert ($keyStoreList.code -eq 0) "keyStoreList failed"

  Info "checking add/update/delete key_store"
  $softwareCode = "VERIFYSTORE" + [DateTimeOffset]::Now.ToUnixTimeSeconds()
  Invoke-MySqlScalar "delete from key_store where key_code = '$softwareCode';" | Out-Null
  $createdAt = "2026-06-28 10:00:00"
  $addStore = Invoke-JsonRequest -Method POST -Url "$BaseUrl/caiji/addKeyStore" -Body @{
    kid                  = 2
    key_code             = $softwareCode
    expiration_date      = 1
    expiration_date_unit = "m"
    is_used              = 0
    remark               = "verify-add"
    created_at           = $createdAt
    updated_at           = $createdAt
  } -WebSession $session -Json
  Info "addKeyStore ok"
  Assert ($addStore.code -eq 0) "addKeyStore failed"

  $storeId = Invoke-MySqlScalar "select id from key_store where key_code = '$softwareCode' limit 1;"
  Assert ($storeId -match '^\d+$') "added key_store not found in db"

  $updateStore = Invoke-JsonRequest -Method POST -Url "$BaseUrl/caiji/updateKeyStore" -Body @{
    id                   = [int]$storeId
    kid                  = 2
    key_code             = $softwareCode
    expiration_date      = 3
    expiration_date_unit = "d"
    is_used              = 1
    remark               = "verify-update"
    created_at           = $createdAt
    updated_at           = "2026-06-28 11:00:00"
  } -WebSession $session -Json
  Info "updateKeyStore ok"
  Assert ($updateStore.code -eq 0) "updateKeyStore failed"

  $storeUpdated = Invoke-MySqlScalar "select concat(expiration_date, ':', expiration_date_unit, ':', is_used, ':', remark) from key_store where id = $storeId;"
  Assert ($storeUpdated -eq "3:d:1:verify-update") "updateKeyStore db verification failed"

  $deleteStore = Invoke-JsonRequest -Method POST -Url "$BaseUrl/caiji/deleteKeyStore" -Body @{ id = [int]$storeId } -WebSession $session -Json
  Info "deleteKeyStore ok"
  Assert ($deleteStore.code -eq 0) "deleteKeyStore failed"

  $storeDeleted = Invoke-MySqlScalar "select count(*) from key_store where id = $storeId;"
  Assert ([int]$storeDeleted -eq 0) "deleteKeyStore db verification failed"

  Info "checking add/update/delete keys_manage"
  $keyCode = "VRFY" + [DateTimeOffset]::Now.ToUnixTimeSeconds()
  $addKey = Invoke-JsonRequest -Method POST -Url "$BaseUrl/caiji/addKeysManage" -Body @{
    key_code   = $keyCode
    user_id    = $null
    expired_at = "2028-12-31 00:00:00"
    is_expired = 0
  } -WebSession $session -Json
  Info "addKeysManage ok"
  Assert ($addKey.code -eq 0) "addKeysManage failed"

  $keyId = Invoke-MySqlScalar "select id from keys_manage where key_code = '$keyCode' limit 1;"
  Assert ($keyId -match '^\d+$') "added key_manage not found in db"

  $updateKey = Invoke-JsonRequest -Method POST -Url "$BaseUrl/caiji/updateKeysManage" -Body @{
    id         = [int]$keyId
    key_code   = $keyCode
    user_id    = $null
    expired_at = "2029-01-31 00:00:00"
    is_expired = 0
  } -WebSession $session -Json
  Info "updateKeysManage ok"
  Assert ($updateKey.code -eq 0) "updateKeysManage failed"

  $updatedExpiredAt = Invoke-MySqlScalar "select expired_at from keys_manage where id = $keyId;"
  Assert ($updatedExpiredAt -match '^2029-01-31') "updateKeysManage db verification failed"

  $deleteKey = Invoke-JsonRequest -Method POST -Url "$BaseUrl/caiji/deleteKeysManage" -Body @{ id = [int]$keyId } -WebSession $session -Json
  Info "deleteKeysManage ok"
  Assert ($deleteKey.code -eq 0) "deleteKeysManage failed"

  Info "checking updateUser"
  $updatedUsername = "verify" + [DateTimeOffset]::Now.ToUnixTimeSeconds()
  $updateUser = Invoke-JsonRequest -Method POST -Url "$BaseUrl/caiji/updateUser" -Body @{
    id         = $sampleUserId
    username   = $updatedUsername
    machine_id = "D843AE52B778"
    expired_at = "2030-01-01 00:00:00"
  } -WebSession $session -Json
  Info "updateUser ok"
  Assert ($updateUser.code -eq 0) "updateUser failed"

  $userExpiredAt = Invoke-MySqlScalar "select expired_at from users where id = $sampleUserId;"
  Assert ($userExpiredAt -match '^2030-01-01') "updateUser db verification failed"

  if ($conflictUsername) {
    $duplicateUser = Invoke-JsonRequest -Method POST -Url "$BaseUrl/caiji/updateUser" -Body @{
      id         = $sampleUserId
      username   = $conflictUsername
      machine_id = "D843AE52B778"
      expired_at = "2030-01-01 00:00:00"
    } -WebSession $session -Json
    Assert ($duplicateUser.code -ne 0) "updateUser duplicate username check failed"
  }

  $restoreUser = Invoke-JsonRequest -Method POST -Url "$BaseUrl/caiji/updateUser" -Body @{
    id         = $sampleUserId
    username   = $sampleUsername
    machine_id = ""
    expired_at = ""
  } -WebSession $session -Json
  Assert ($restoreUser.code -eq 0) "updateUser restore failed"

  Info "checking add/delete caiji"
  $caijiName = "codex-verify-" + [DateTimeOffset]::Now.ToUnixTimeSeconds()
  $addCaiji = Invoke-JsonRequest -Method POST -Url "$BaseUrl/caiji/addCaiji" -Body @{ cj_name = $caijiName } -WebSession $session -Json
  Info "addCaiji ok"
  Assert ($addCaiji.code -eq 0) "addCaiji failed"

  Start-Sleep -Milliseconds 500
  $caijiId = Invoke-MySqlScalar "select id from dy_caiji where cj_name = '$caijiName' limit 1;"
  Assert ($caijiId -match '^\d+$') "added caiji not found in db"

  $deleteCaiji = Invoke-JsonRequest -Method POST -Url "$BaseUrl/caiji/deleteCaiji" -Body @{ id = @([int]$caijiId) } -WebSession $session -Json
  Info "deleteCaiji ok"
  Assert ($deleteCaiji.code -eq 0) "deleteCaiji failed"

  $deleteCookie = Invoke-JsonRequest -Method POST -Url "$BaseUrl/caiji/deleteCookie" -Body @{ id = @([int]$cookieId) } -WebSession $session -Json
  Info "deleteCookie ok"
  Assert ($deleteCookie.code -eq 0) "deleteCookie failed"

  $cookieDeleted = Invoke-MySqlScalar "select count(*) from dy_cookies where id = $cookieId;"
  Assert ([int]$cookieDeleted -eq 0) "deleteCookie db verification failed"

  Info "checking allocate cookie"
  $allocate = Invoke-JsonRequest -Method POST -Url "$BaseUrl/caiji/allocateCookie" -Body @{ caiji = @($sampleCaijiId); cookie = @(1, 2) } -WebSession $session -Json
  Info "allocateCookie ok"
  Assert ($allocate.code -eq 0) "allocateCookie failed"
  $bindCount = Invoke-MySqlScalar "select count(*) from dy_cj_ck where cj_id = $sampleCaijiId and ck_id in (1,2);"
  Assert ([int]$bindCount -ge 2) "allocateCookie db verification failed"

  Info "checking client api routes"
  $headers = @{ "app-key" = $AppKey }
  $roomUrl = "https://live.douyin.com/verify-" + [DateTimeOffset]::Now.ToUnixTimeSeconds()
  $addRoom = Invoke-JsonRequest -Method POST -Url "$BaseUrl/api/addRoom" -Body @{
    room_url   = $roomUrl
    room_id    = "verify-room"
    web_rid    = "verify-web-rid"
    title      = "验证房间"
    tp         = "hb"
    send_time  = 1782655800
    delay      = 300
    expired_at = 1782656100
  } -Headers $headers -Json
  Info "addRoom ok"
  Assert ($addRoom.code -eq 0) "addRoom failed"

  $fetchRoom = Invoke-JsonRequest -Method POST -Url "$BaseUrl/api/fetchRoom" -Body @{
    type   = "hb"
    params = @{ can_used_times = 100 }
  } -Headers $headers -Json
  Info "fetchRoom ok"
  Assert ($fetchRoom.code -eq 0) "fetchRoom failed"

  $fetchOneRoom = Invoke-JsonRequest -Method GET -Url "$BaseUrl/api/fetchOneRoom?type=hb" -Headers $headers
  Info "fetchOneRoom ok"
  Assert ($fetchOneRoom.code -eq 0) "fetchOneRoom failed"

  $updateDailyLog = Invoke-JsonRequest -Method POST -Url "$BaseUrl/api/updateDailyLog" -Body @{
    nickname = "验证用户"
    dy_no    = "verify-dy-no"
    type     = "hb"
    hb_z_zs  = 1
    hb_hd_zs = 5
    hb_xh_zs = 2
    ss_zs    = 20
  } -Headers $headers -Json
  Info "updateDailyLog ok"
  Assert ($updateDailyLog.code -eq 0) "updateDailyLog failed"

  $dailyRow = Invoke-MySqlScalar "select count(*) from dy_daily_log where dy_no = 'verify-dy-no';"
  Assert ([int]$dailyRow -ge 1) "updateDailyLog db verification failed"

  Info "checking recharge"
  Invoke-MySqlScalar "delete from key_store where key_code='VERIFYRECHARGE01';"
  Invoke-MySqlScalar "insert into key_store (kid,key_code,expiration_date,expiration_date_unit,is_used,remark,created_at) values (2,'VERIFYRECHARGE01',1,'m',0,'verify',now());"
  Invoke-MySqlScalar "update keys_manage set expired_at='2027-06-11 17:03:41' where id=2;"

  $recharge = Invoke-JsonRequest -Method POST -Url "$BaseUrl/auth/recharge" -Body @{
    username    = "321@qq.com"
    keycode     = "VERIFYRECHARGE01"
    mac_address = "E89C2510F9F5"
  }
  Info "recharge ok"
  Assert ($recharge.code -eq 0) "recharge failed"

  $rechargeUsed = Invoke-MySqlScalar "select is_used from key_store where key_code='VERIFYRECHARGE01';"
  Assert ([int]$rechargeUsed -eq 1) "recharge did not mark key_store used"

  Write-Host ""
  Write-Host "verify.ps1 passed"
} finally {
  Stop-Server $server
}
