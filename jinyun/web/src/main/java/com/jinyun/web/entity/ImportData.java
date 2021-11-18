package com.jinyun.web.entity;

import lombok.AllArgsConstructor;
import lombok.Data;
import lombok.NoArgsConstructor;

@Data
@AllArgsConstructor
@NoArgsConstructor
public class ImportData {
    String stationName;
    String lineName;
    String importTime;
}
