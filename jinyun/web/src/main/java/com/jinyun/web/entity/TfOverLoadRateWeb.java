package com.jinyun.web.entity;

import lombok.AllArgsConstructor;
import lombok.Data;
import lombok.NoArgsConstructor;

@Data
@AllArgsConstructor
@NoArgsConstructor
public class TfOverLoadRateWeb {
    String feederName;
    String devName;
    String mRID;
    String lineName;
    String lineMRID;
    String substation;
    double springRate;
    double summerRate;
    double autumnRate;
    double winterRate;
}
