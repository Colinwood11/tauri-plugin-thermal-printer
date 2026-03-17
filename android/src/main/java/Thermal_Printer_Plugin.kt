package com.luis3132.thermal_printer

import android.app.Activity
import android.util.Log
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import app.tauri.plugin.Invoke
import app.tauri.annotation.Permission
import android.Manifest
import android.os.Build

@TauriPlugin(
    permissions = [
        Permission(
            strings = [
                Manifest.permission.BLUETOOTH,
                Manifest.permission.BLUETOOTH_ADMIN,
                Manifest.permission.BLUETOOTH_SCAN,
                Manifest.permission.BLUETOOTH_CONNECT,
                Manifest.permission.ACCESS_FINE_LOCATION,
                Manifest.permission.ACCESS_COARSE_LOCATION
            ],
            alias = "bluetooth"
        )
    ]
)
class Thermal_Printer_Plugin(private val activity: Activity) : Plugin(activity) {

    private val TAG = "ThermalPrinterPlugin"

    /**
     * Lista todas las impresoras térmicas disponibles.
     * NOTA: @Command NO puede ser suspend — Tauri Android no soporta corrutinas directamente.
     * Las operaciones de red se deben hacer en un hilo separado.
     */
    @Command
    fun list_thermal_printers(invoke: Invoke) {
        Log.d(TAG, "list_thermal_printers called")

        // Ejecutar en hilo secundario para no bloquear el hilo principal
        Thread {
            try {
                val discovery = PrinterDiscovery(activity.applicationContext)
                val printers = discovery.discoverAllPrinters()

                Log.d(TAG, "Total printers found: ${printers.size}")

                // Construir JSArray — esto es lo que Tauri deserializa como Vec<PrinterInfo> en Rust
                val printersArray = JSArray()
                for (printer in printers) {
                    val obj = JSObject().apply {
                        // Los nombres deben coincidir EXACTAMENTE con los campos del struct
                        // PrinterInfo en models.rs (snake_case por convención Rust/serde)
                        put("name", printer.name)
                        put("interface_type", printer.interfaceType)   // snake_case para serde
                        put("identifier", printer.identifier)
                        put("status", printer.status)
                    }
                    printersArray.put(obj)
                }

                val result = JSObject()
                // run_mobile_plugin en Rust espera que el resultado sea el valor raíz,
                // o un objeto con los campos del tipo de retorno.
                // Para Vec<PrinterInfo>, Tauri espera un array JSON directamente,
                // pero run_mobile_plugin lo envuelve — ponemos "printers" si el modelo
                // en Rust es un wrapper, o resolvemos directamente si es Vec plano.
                // Basado en tu código Rust: Ok(self.0.run_mobile_plugin("list_thermal_printers", ())?)
                // Tauri deserializa la respuesta como Vec<PrinterInfo>, por lo que necesitamos
                // devolver el array en la clave que Tauri usa internamente.
                // La forma correcta es poner el array en la respuesta directamente:
                result.put("printers", printersArray)

                Log.d(TAG, "Resolving with ${printers.size} printers")
                invoke.resolve(result)

            } catch (e: Exception) {
                Log.e(TAG, "Error listing printers: ${e.message}", e)
                invoke.reject("Error listing printers: ${e.message}")
            }
        }.start()
    }
}