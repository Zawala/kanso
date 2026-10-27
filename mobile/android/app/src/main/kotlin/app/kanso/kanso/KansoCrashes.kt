package app.kanso.kanso

import android.app.ActivityManager
import android.app.ApplicationExitInfo
import android.content.Context
import android.os.Build
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel
import java.io.File
import java.io.InputStream
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale
import java.util.TimeZone
import org.json.JSONArray
import org.json.JSONObject

/**
 * Crash reports on Android (docs/CRASH-REPORTS.md).
 *
 * This only drops raw crashes into `files/crash-reports/raw/`, the folder
 * `getApplicationSupportDirectory()` names plus `crash-reports/raw`. The Dart
 * side (`mobile/lib/crash/`) reads them, keeps only what the whitelist lets
 * through, and deletes them. Nothing is sent from here, and the Dart side
 * ignores `*.tmp` while a file is being written.
 *
 * Three sources:
 *  - a native crash (`REASON_CRASH_NATIVE`): the tombstone, as the protobuf the
 *    system keeps, `<timestamp>.tombstone.pb`;
 *  - an ANR (`REASON_ANR`): the text trace, `<timestamp>.anr.txt`;
 *  - an uncaught Kotlin or Java exception: `<timestamp>.jvm.json` with the
 *    class name and the frames. The exception's message is never read.
 *
 * `files/` is out of backups: the backup rules exclude every domain.
 */
object KansoCrashes {
    /** The method channel the Dart side asks for the device model on. */
    const val CHANNEL = "app.kanso/crash"

    private const val PREFERENCES = "kanso_crashes"
    private const val LAST_EXIT = "last_exit_timestamp"
    private const val MAX_FRAMES = 128
    private const val MAX_RAW_BYTES = 16 * 1024 * 1024
    private const val MAX_AGE_MS = 14L * 24 * 60 * 60 * 1000

    /** Call from `Application.onCreate`: the handler, and the earlier exits. */
    @JvmStatic
    fun start(context: Context) {
        installUncaughtExceptionHandler(context)
        // Only the app's own process; a background job's process would
        // collect the same exits twice.
        if (processName() == context.packageName) {
            collectEarlierExits(context)
        }
    }

    /** Answers `deviceModel` and `osVersion` on [CHANNEL]. */
    @JvmStatic
    fun register(engine: FlutterEngine) {
        MethodChannel(engine.dartExecutor.binaryMessenger, CHANNEL)
            .setMethodCallHandler { call, result ->
                when (call.method) {
                    "deviceModel" -> result.success("${Build.MANUFACTURER} ${Build.MODEL}")
                    "osVersion" -> result.success(Build.VERSION.RELEASE)
                    else -> result.notImplemented()
                }
            }
    }

    @Suppress("FunctionName")
    private fun processName(): String? = android.app.Application.getProcessName()

    private fun rawFolder(context: Context): File {
        val folder = File(context.filesDir, "crash-reports/raw")
        folder.mkdirs()
        return folder
    }

    private fun installUncaughtExceptionHandler(context: Context) {
        val previous = Thread.getDefaultUncaughtExceptionHandler()
        val appContext = context.applicationContext
        Thread.setDefaultUncaughtExceptionHandler { thread, throwable ->
            try {
                writeException(appContext, throwable)
            } catch (_: Throwable) {
                // A report that can't be written must not hide the crash.
            }
            previous?.uncaughtException(thread, throwable)
        }
    }

    private fun writeException(context: Context, throwable: Throwable) {
        val frames = JSONArray()
        for (element in throwable.stackTrace.take(MAX_FRAMES)) {
            frames.put(
                JSONObject()
                    .put("cls", element.className)
                    .put("method", element.methodName)
                    .put("line", element.lineNumber)
            )
        }
        val format = SimpleDateFormat("yyyy-MM-dd", Locale.US)
        format.timeZone = TimeZone.getTimeZone("UTC")
        // The class name and the frames only; never the message.
        val json = JSONObject()
            .put("type", throwable.javaClass.name)
            .put("frames", frames)
            .put("day", format.format(Date()))
        val now = System.currentTimeMillis()
        writeAtomically(rawFolder(context), "$now.jvm.json") { out ->
            out.write(json.toString().toByteArray(Charsets.UTF_8))
        }
    }

    private fun collectEarlierExits(context: Context) {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.R) {
            return
        }
        val appContext = context.applicationContext
        Thread {
            try {
                val preferences =
                    appContext.getSharedPreferences(PREFERENCES, Context.MODE_PRIVATE)
                val last = preferences.getLong(LAST_EXIT, 0L)
                val manager = appContext.getSystemService(ActivityManager::class.java)
                val exits = manager.getHistoricalProcessExitReasons(appContext.packageName, 0, 5)
                var newest = last
                val oldest = System.currentTimeMillis() - MAX_AGE_MS
                for (info in exits) {
                    if (info.timestamp <= last) {
                        continue
                    }
                    newest = maxOf(newest, info.timestamp)
                    if (info.timestamp < oldest) {
                        continue
                    }
                    val suffix = when (info.reason) {
                        ApplicationExitInfo.REASON_CRASH_NATIVE -> "tombstone.pb"
                        ApplicationExitInfo.REASON_ANR -> "anr.txt"
                        else -> null
                    } ?: continue
                    val stream = info.traceInputStream ?: continue
                    stream.use { input ->
                        writeAtomically(
                            rawFolder(appContext),
                            "${info.timestamp}.$suffix"
                        ) { out -> copyLimited(input, out) }
                    }
                }
                preferences.edit().putLong(LAST_EXIT, newest).commit()
            } catch (_: Throwable) {
                // Earlier exits are a nicety; the app starts either way.
            }
        }.start()
    }

    private fun copyLimited(input: InputStream, out: java.io.OutputStream) {
        val buffer = ByteArray(8192)
        var total = 0
        while (total < MAX_RAW_BYTES) {
            val read = input.read(buffer)
            if (read < 0) {
                break
            }
            out.write(buffer, 0, read)
            total += read
        }
    }

    /** Writes to `<name>.tmp` and renames, so the Dart side never sees half a file. */
    private fun writeAtomically(
        folder: File,
        name: String,
        write: (java.io.OutputStream) -> Unit
    ) {
        val temporary = File(folder, "$name.tmp")
        temporary.outputStream().use(write)
        if (!temporary.renameTo(File(folder, name))) {
            temporary.delete()
        }
    }
}
