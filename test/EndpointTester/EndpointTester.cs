// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// ENDPOINT TESTER UTILITY
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

using System;
using NAudio.CoreAudioApi;

namespace EndpointTester
{
    class Program
    {
        static void Main(string[] args)
        {
            Console.WriteLine("=== Leyline Audio Driver Endpoint Tester ===");
            Console.WriteLine($"Starting enumeration at {DateTime.Now}...\n");

            var enumerator = new MMDeviceEnumerator();
            Console.WriteLine("--- Render Endpoints ---");
            EnumerateEndpoints(enumerator, DataFlow.Render);

            Console.WriteLine("\n--- Capture Endpoints ---");
            EnumerateEndpoints(enumerator, DataFlow.Capture);

            Console.WriteLine("\nEnumeration complete.");
            Console.ReadLine();
        }

        static void EnumerateEndpoints(MMDeviceEnumerator enumerator, DataFlow dataFlow)
        {
            var devices = enumerator.EnumerateAudioEndPoints(dataFlow, DeviceState.All);
            int count = 0;
            foreach (var device in devices)
            {
                if (device.FriendlyName.Contains("Leyline") || device.DeviceInterfaceFriendlyName.Contains("Leyline"))
                {
                    Console.WriteLine($"[FOUND] {device.FriendlyName} ({device.State})");
                    Console.WriteLine($"        ID: {device.ID}");
                    count++;
                }
            }

            if (count == 0)
            {
                Console.WriteLine("        No Leyline endpoints found.");
            }
        }
    }
}

