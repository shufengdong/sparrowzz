package com.jinyun.web.controller;

import com.jinyun.web.service.CapService;
import io.swagger.annotations.Api;
import io.swagger.annotations.ApiImplicitParam;
import io.swagger.annotations.ApiImplicitParams;
import io.swagger.annotations.ApiOperation;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RequestMethod;
import org.springframework.web.bind.annotation.RequestParam;
import org.springframework.web.bind.annotation.RestController;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

@Api(value = "desc of class", tags="LowMarginWarn",description = "低裕度预警分析")
@RestController
@RequestMapping("lowMarginWarn")
public class LowMarginWarnController {
    @Autowired
    CapService capService;

    @ApiOperation(value = "lowMarginAnalysis", notes = "")
    @RequestMapping(value = "/lowMarginAnalysis",method = RequestMethod.GET)
    public Object lowMarginAnalysis() {
        Map<String, Object> result = capService.lowMarginAnalysis();
        return result;
    }

}
