package dev.bwt.libbwt.daemon

import com.google.gson.Gson
import com.google.gson.annotations.SerializedName
import dev.bwt.libbwt.BwtException
import java.util.*

class BwtDaemon(
    var config: BwtConfig,
) {
    var started: Boolean = false
    var ready: Boolean = false
    var terminate: Boolean = false
    var shutdownPtr: Long? = null
    var electrumAddr: String? = null
    var httpAddr: String? = null

    fun start(callback: ProgressNotifier) {
        if (started) throw BwtException("daemon already started")
        started = true
        val jsonConfig = Gson().toJson(config)
        NativeBwtDaemon.start(jsonConfig, object : CallbackNotifier {
            override fun onBooting(shutdownPtr_: Long) {
                shutdownPtr = shutdownPtr_
                if (!terminate) callback.onBooting()
                else shutdown()
            }

            override fun onSyncProgress(progress: Float, tipUnix: Int) {
                val tipDate = Date(tipUnix.toLong() * 1000)
                if (!ready && !terminate) callback.onSyncProgress(progress, tipDate)
            }

            override fun onScanProgress(progress: Float, eta: Int) {
                if (!ready && !terminate) callback.onScanProgress(progress, eta)
            }

            override fun onElectrumReady(addr: String) {
                electrumAddr = addr
            }

            override fun onHttpReady(addr: String) {
                httpAddr = addr
            }

            override fun onReady() {
                ready = true
                if (!terminate) callback.onReady()
            }
        })
    }

    fun shutdown() {
        // If we don't have the shutdownPtr yet, this will mark the daemon for later termination
        terminate = true

        shutdownPtr?.let { ptr ->
            shutdownPtr = null
            NativeBwtDaemon.shutdown(ptr)
        }
    }
}

interface ProgressNotifier {
    fun onBooting() {};
    fun onSyncProgress(progress: Float, tip: Date) {};
    fun onScanProgress(progress: Float, eta: Int) {};
    fun onReady() {};
}

data class BwtConfig(
    @SerializedName("network") var network: String? = null,
    @SerializedName("bitcoind_url") var bitcoindUrl: String? = null,
    @SerializedName("bitcoind_auth") var bitcoindAuth: String? = null,
    @SerializedName("bitcoind_dir") var bitcoindDir: String? = null,
    @SerializedName("bitcoind_cookie") var bitcoindCookie: String? = null,
    @SerializedName("bitcoind_wallet") var bitcoindWallet: String? = null,
    @SerializedName("descriptors") var descriptors: Array<String>? = null,
    @SerializedName("xpubs") var xpubs: Array<String>? = null,
    @SerializedName("rescan_since") var rescanSince: Int? = null,
    @SerializedName("gap_limit") var gapLimit: Int? = null,
    @SerializedName("initial_import_size") var initialImportSize: Int? = null,
    @SerializedName("poll_interval") var pollInterval: Array<Int>? = null,
    @SerializedName("verbose") var verbose: Int? = null,
    @SerializedName("tx_broadcast_cmt") var txBroadcastCmd: String? = null,
    @SerializedName("electrum_addr") var electrumAddr: String? = null,
    @SerializedName("electrum_skip_merkle") var electrumSkipMerkle: Boolean? = null,
    @SerializedName("http_addr") var httpAddr: String? = null,
    @SerializedName("http_cors") var httpCors: Boolean? = null,
    @SerializedName("webhooks_urls") var webhookUrls: Array<String>? = null,
    @SerializedName("unix_listener_path") var unixListenerPath: String? = null,
    @SerializedName("require_addresses") var requireAddresses: Boolean? = null,
    @SerializedName("setup_logger") var setupLogger: Boolean? = null,
    @SerializedName("force_rescan") var forceRescan: Boolean? = null,
) {}