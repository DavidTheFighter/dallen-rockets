#include <NativeEthernet.h>

#include <imxrt_flexcan.h>
#include <circular_buffer.h>
#include <kinetis_flexcan.h>
#include <FlexCAN_T4.h>

#define MAX_SERIALIZE_LENGTH 60

struct CommsMessage {
  uint16_t from_address;
  uint16_t to_address;
  uint16_t data_len;
  uint8_t data[MAX_SERIALIZE_LENGTH];
};

FlexCAN_T4FD<CAN3, RX_SIZE_256, TX_SIZE_16> can3;

byte mac[] = {
  0xDE, 0xAD, 0xBE, 0xEF, 0xFE, 0xED
};
IPAddress ip(10, 0, 0, 5);
unsigned int localPort = 8888;

IPAddress mcctrlIP(10, 0, 0, 4);
unsigned int mcctrlPort = 25565;

EthernetUDP udp;

void onReceive(const CANFD_message_t &msg);
void panic();

void setup() {
  Ethernet.begin(mac, ip);
  
  Serial.begin(9600);
  
  pinMode(LED_BUILTIN, OUTPUT);
  
  CANFD_timings_t config;
  config.clock = CLK_30MHz;
  config.baudrate = 1000000;
  config.baudrateFD = 2000000;
  config.propdelay = 190;
  config.bus_length = 1;
  config.sample = 70;

  CANFD_exact_timings_t customConfig;
  customConfig.prop_seg = 13;
  customConfig.phase_seg_1 = 3;
  customConfig.phase_seg_2 = 3;
  customConfig.jump_width = 3;
  customConfig.prescalar_division = 1;
  
  customConfig.fd_prop_seg = 1;
  customConfig.fd_phase_seg_1 = 3;
  customConfig.fd_phase_seg_2 = 2;
  customConfig.fd_jump_width = 2;
  customConfig.fd_prescalar_division = 1;
  
  customConfig.tdcoff = 3;

  can3.begin();
  can3.setBaudRate(config);
  can3.setRegions(64);
  can3.enableMBInterrupts();
  can3.onReceive(onReceive);
  can3.setExactTimings(customConfig);

  if (Ethernet.hardwareStatus() == EthernetNoHardware) {
    Serial.println("Ethernet shield was not found.  Sorry, can't run without hardware. :(");
    panic();
  }
  
  if (Ethernet.linkStatus() == LinkOFF) {
    Serial.println("Ethernet cable is not connected.");
  }

  // start UDP
  if (!udp.begin(localPort)) {
    Serial.println("Failed to start UDP");
    panic();
  }

  if (sizeof(CommsMessage) != MAX_SERIALIZE_LENGTH + 6) {
    Serial.print("Size is incorrect: ");
    Serial.println(sizeof(CommsMessage));

    panic();
  }

  Serial.println("Done!");
}

void sendPacket(CommsMessage &msg) {
  udp.beginPacket(IPAddress(10, 0, 0, 255), mcctrlPort);
  udp.write(reinterpret_cast<const uint8_t*>(&msg), sizeof(CommsMessage));
  udp.endPacket(); 
}

#define FROM_ADDRESS_SHIFT 6
#define ID_5_BIT_MASK 0x1F
#define TRUE_LENGTH_MASK 0x3F

void onReceive(const CANFD_message_t &msg) {
  uint32_t metadata = uint32_t(msg.buf[0]) 
    + (uint32_t(msg.buf[1]) << 8) 
    + (uint32_t(msg.buf[2]) << 16) 
    + (uint32_t(msg.buf[3]) << 24);

  CommsMessage send = {};
  send.from_address = (msg.id >> FROM_ADDRESS_SHIFT) & ID_5_BIT_MASK;
  send.to_address = msg.id & ID_5_BIT_MASK;
  send.data_len = constrain(metadata & TRUE_LENGTH_MASK, 1, MAX_SERIALIZE_LENGTH);
  
  memcpy(send.data, &msg.buf[4], sizeof(send.data));

  sendPacket(send);
}

#define INCOMING_BUFFER_SIZE 512

uint8_t incomingBuffer[INCOMING_BUFFER_SIZE];

uint32_t counter = 0;
uint32_t flip = 0;

void loop() {
  delayMicroseconds(10);

  if (counter % 100000 == 0) {
    digitalWrite(LED_BUILTIN, (flip++) % 2);
    Serial.println("PULSE");
  }

  if (counter % 10000 == 0) {
    CommsMessage send = {};
    send.from_address = 255;
    send.to_address = 255;

    sendPacket(send);
  }

  counter ++;

  int packetSize;
  while ((packetSize = udp.parsePacket()) != 0) {
    Serial.println("Got packet");
    udp.read(incomingBuffer, INCOMING_BUFFER_SIZE);

    CommsMessage recv;
    memcpy(&recv, incomingBuffer, sizeof(recv));

    if (recv.from_address == 65535 && recv.to_address == 65535) {
      mcctrlIP = udp.remoteIP();
      Serial.print("Set mission control (remote IP) to: ");
      mcctrlIP.printTo(Serial);
      Serial.print("\n");
      continue;
    }

    if (recv.data_len == 0) {
      continue;
    }

    uint32_t ctrl = 0;
    ctrl += constrain(recv.data_len, 1, MAX_SERIALIZE_LENGTH);

    CANFD_message_t msg;
    msg.id = recv.to_address & ID_5_BIT_MASK;
    msg.id += (recv.from_address & ID_5_BIT_MASK) << FROM_ADDRESS_SHIFT;
    msg.id += (recv.data_len + 4 > 32 ? 1 : 0) << 5;
    msg.len = recv.data_len + 4;
    memcpy(&msg.buf[0], &ctrl, sizeof(ctrl));
    memcpy(&msg.buf[4], &incomingBuffer[4], recv.data_len);

    can3.write(msg);
  }

  Ethernet.maintain();
  while (can3.events() != 0);
}

void panic() {
  while (true) {
    digitalWrite(LED_BUILTIN, !digitalRead(LED_BUILTIN));

    delay(100);
  }
}