package com.chunkedge.extractor.extractors;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.chunkedge.extractor.Main;
import java.io.IOException;
import net.minecraft.network.ProtocolInfo;
import net.minecraft.network.protocol.configuration.ConfigurationProtocols;
import net.minecraft.network.protocol.game.GameProtocols;
import net.minecraft.network.protocol.handshake.HandshakeProtocols;
import net.minecraft.network.protocol.login.LoginProtocols;
import net.minecraft.network.protocol.status.StatusProtocols;

public class Packets implements Main.Extractor {

    @Override
    public String fileName() {
        return "packets.json";
    }

    @Override
    public JsonElement extract() throws IOException {
        var packetsJson = new JsonArray();

        serialize(HandshakeProtocols.SERVERBOUND_TEMPLATE, packetsJson);
        serialize(StatusProtocols.SERVERBOUND_TEMPLATE, packetsJson);
        serialize(StatusProtocols.CLIENTBOUND_TEMPLATE, packetsJson);
        serialize(LoginProtocols.SERVERBOUND_TEMPLATE, packetsJson);
        serialize(LoginProtocols.CLIENTBOUND_TEMPLATE, packetsJson);
        serialize(ConfigurationProtocols.SERVERBOUND_TEMPLATE, packetsJson);
        serialize(ConfigurationProtocols.CLIENTBOUND_TEMPLATE, packetsJson);
        // NOTE: the game (play) templates are unbound (they require a
        // registry-aware buffer + context to encode/decode), but packet ID
        // enumeration does not touch the codec, so `details()` is enough.
        serialize(GameProtocols.SERVERBOUND_TEMPLATE, packetsJson);
        serialize(GameProtocols.CLIENTBOUND_TEMPLATE, packetsJson);

        return packetsJson;
    }

    private static void serialize(
        ProtocolInfo.DetailsProvider protocol,
        JsonArray json
    ) {
        var details = protocol.details();
        details.listPackets((type, i) -> {
            var packetJson = new JsonObject();
            packetJson.addProperty("name", type.id().getPath());
            packetJson.addProperty("phase", details.id().id());
            packetJson.addProperty("side", details.flow().id());
            packetJson.addProperty("id", i);
            json.add(packetJson);
        });
    }
}
