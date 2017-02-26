#include <Arduino.h>
#include <ESP8266WiFi.h>
#include <WiFiUdp.h>
#include <Servo.h>

// Maximum size of packets to hold in the buffer
#define BUFFER_LEN 1024

// Wifi and socket settings
const char* ssid     = "";
const char* password = "";
unsigned int localPort = 37210;
char packetBuffer[BUFFER_LEN];

#define LEFT_PIN  5
#define RIGHT_PIN 4

WiFiUDP port;

// Set this to something sensible
IPAddress ip(0,0,0,0);
IPAddress gateway(10,1,1,1);
IPAddress subnet(255,0,0,0);

Servo left;
Servo right;

void setup() {
    Serial.begin(115200);
    WiFi.config(ip, gateway, subnet);
    WiFi.begin(ssid, password);
    while (WiFi.status() != WL_CONNECTED) {
        delay(500);
        Serial.print(".");
    }
    Serial.print("Connected to ");
    Serial.println(ssid);
    Serial.print("IP address: ");
    Serial.println(WiFi.localIP());
    port.begin(localPort);

    pinMode(LEFT_PIN , OUTPUT);
    pinMode(RIGHT_PIN, OUTPUT);

    left.attach(LEFT_PIN);
    right.attach(RIGHT_PIN);
}

// 16bit frame number + two 16 bit servo points
#define PACKET_SIZE (3 * sizeof(uint16_t))

// if more than this amount of frames out of sync, resync
// do not exceed 2^15-1
#define FRAME_TOLERANCE 300

uint16_t last_frame = 0xFFFF;
void loop() {
  
    // Read data over socket
    int packetSize = port.parsePacket();
    // If packets have been received, interpret the command
    if (packetSize) {
        int len = port.read(packetBuffer, BUFFER_LEN);
        if(len == PACKET_SIZE){ // len must be match the packet size
          uint16_t incoming_frame = packetBuffer[0] << 8 | packetBuffer[1];
          int32_t diff = (int32_t)(incoming_frame - last_frame);
          if(diff > 0 || diff < -FRAME_TOLERANCE){
            uint16_t left_value  = packetBuffer[2] << 8 | packetBuffer[3];
            uint16_t right_value = packetBuffer[4] << 8 | packetBuffer[5];
            left.write (left_value );
            right.write(right_value);
            Serial.printf("left %u right %u\n", left_value, right_value);
          }
       }
    }
}
