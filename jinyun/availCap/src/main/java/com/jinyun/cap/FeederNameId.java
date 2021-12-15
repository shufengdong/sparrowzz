package com.jinyun.cap;

import java.util.HashMap;
import java.util.Map;

public class FeederNameId {
    Map<Integer, String> feederIdToName = new HashMap<>();
    Map<String, Integer> feederNameToId = new HashMap<>();

    public Map<Integer, String> getFeederIdToName() {
        return feederIdToName;
    }

    public void setFeederIdToName(Map<Integer, String> feederIdToName) {
        this.feederIdToName = feederIdToName;
    }

    public Map<String, Integer> getFeederNameToId() {
        return feederNameToId;
    }

    public void setFeederNameToId(Map<String, Integer> feederNameToId) {
        this.feederNameToId = feederNameToId;
    }
}
