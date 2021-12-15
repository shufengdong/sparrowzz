package com.jinyun.cap;

import java.util.HashMap;
import java.util.Map;

public class FeederNameId {
    Map<String, String> feederIdToName = new HashMap<>();
    Map<String, String> feederNameToId = new HashMap<>();

    public Map<String, String> getFeederIdToName() {
        return feederIdToName;
    }

    public void setFeederIdToName(Map<String, String> feederIdToName) {
        this.feederIdToName = feederIdToName;
    }

    public Map<String, String> getFeederNameToId() {
        return feederNameToId;
    }

    public void setFeederNameToId(Map<String, String> feederNameToId) {
        this.feederNameToId = feederNameToId;
    }
}
