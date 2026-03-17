package com.luis3132.thermal_printer

import android.Manifest
import android.bluetooth.BluetoothAdapter
import android.bluetooth.BluetoothManager
import android.bluetooth.BluetoothSocket
import android.content.Context
import android.content.pm.PackageManager
import android.os.Build
import android.util.Log
import androidx.core.app.ActivityCompat
import java.util.UUID

class BluetoothPrinter(private val context: Context) {

    private val TAG = "BluetoothPrinter"
    private val SPP_UUID = UUID.fromString("00001101-0000-1000-8000-00805F9B34FB")

    private val bluetoothAdapter: BluetoothAdapter? by lazy {
        val manager = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
            context.getSystemService(BluetoothManager::class.java)
        } else {
            @Suppress("DEPRECATION")
            context.getSystemService(Context.BLUETOOTH_SERVICE) as? BluetoothManager
        }
        manager?.adapter
    }

    fun printRawData(macAddress: String, data: ByteArray) {
        val adapter = bluetoothAdapter
            ?: throw IllegalStateException("Bluetooth adapter not available")

        if (!adapter.isEnabled) {
            throw IllegalStateException("Bluetooth is disabled")
        }

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            if (ActivityCompat.checkSelfPermission(
                    context,
                    Manifest.permission.BLUETOOTH_CONNECT
                ) != PackageManager.PERMISSION_GRANTED
            ) {
                throw SecurityException("BLUETOOTH_CONNECT permission not granted")
            }
        }

        Log.d(TAG, "Connecting to $macAddress (${data.size} bytes)")
        val device = adapter.getRemoteDevice(macAddress)
        adapter.cancelDiscovery()

        var socket: BluetoothSocket? = null
        try {
            socket = device.createRfcommSocketToServiceRecord(SPP_UUID)
            socket.connect()
            Log.d(TAG, "Connected, sending ${data.size} bytes")
            socket.outputStream.write(data)
            socket.outputStream.flush()
            Log.d(TAG, "Print complete")
        } finally {
            try { socket?.close() } catch (_: Exception) {}
        }
    }
}
