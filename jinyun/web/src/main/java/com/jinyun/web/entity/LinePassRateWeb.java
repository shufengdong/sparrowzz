package com.jinyun.web.entity;

import lombok.AllArgsConstructor;
import lombok.Data;
import lombok.NoArgsConstructor;

@Data
@AllArgsConstructor
@NoArgsConstructor
public class LinePassRateWeb {
    String feederName;
    String substation;
    double springRate;
    double summerRate;
    double autumnRate;
    double winterRate;
}
