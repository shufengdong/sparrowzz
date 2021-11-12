package com.jinyun.web.controller;

import com.jinyun.web.service.CapService;
import io.swagger.annotations.Api;
import io.swagger.annotations.ApiOperation;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.web.bind.annotation.*;

import java.util.Map;

@Api(value = "desc of class", tags="MainPage",description = "首页")
@RestController
@RequestMapping("mainPage")
public class MainPageController {
    @Autowired
    CapService capService;

    @ApiOperation(value = "电网统计字段", notes = "")
    @RequestMapping(value = "/gridStatistics",method = RequestMethod.GET)
    public Object gridStatistics() {
        Map<String,Object> result = capService.gridStatistics();
        return result;
    }

    @ApiOperation(value = "低裕度预警统计", notes = "")
    @RequestMapping(value = "/lowMarginWarnStatistics",method = RequestMethod.GET)
    public Object lowMarginWarnStatistics() {
        Map<String,Object> result = capService.lowMarginWarnStatistics();
        return result;
    }

    @ApiOperation(value = "配变三相不平衡度", notes = "")
    @RequestMapping(value = "/transformerUnbalance",method = RequestMethod.GET)
    public Object transformerUnbalance() {
        Map<String,Object> result = capService.transformerUnbalance();
        return result;
    }

    @ApiOperation(value = "配变三相不平衡度列表", notes = "")
    @RequestMapping(value = "/transformerUnbalanceList",method = RequestMethod.GET)
    public Object transformerUnbalanceList() {
        Map<String,Object> result = capService.transformerUnbalanceList();
        return result;
    }


}
